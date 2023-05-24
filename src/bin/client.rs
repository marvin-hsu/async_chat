use std::sync::Arc;

use async_chat::{
    utils::{self, ChatResult},
    FromClient, FromServer,
};
use async_std::{
    io::{self, prelude::BufReadExt, WriteExt},
    net,
    prelude::FutureExt,
    stream::StreamExt,
    task,
};

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: client ADDRESS:PORT");
    task::block_on(async {
        let socket = net::TcpStream::connect(address).await?;
        socket.set_nodelay(true)?;

        let to_server = send_commands(socket.clone());
        let from_server = handle_replies(socket);

        from_server.race(to_server).await?;
        Ok(())
    })
}

async fn handle_replies(from_server: net::TcpStream) -> ChatResult<()> {
    let buffered = io::BufReader::new(from_server);
    let mut reply_stream = utils::receive_as_json(buffered);

    while let Some(reply) = reply_stream.next().await {
        match reply? {
            FromServer::Message {
                group_name,
                message,
            } => {
                println!("message posted to {}: {}", group_name, message);
            }
            FromServer::Error(message) => {
                println!("error from server: {}", message);
            }
        }
    }

    Ok(())
}

async fn send_commands(mut to_servier: net::TcpStream) -> ChatResult<()> {
    println!("Command:\njoin GROUP\npost GROUP MESSAGE...\nType Control-D (on Unix) or Control-Z (on Windows) to close the connection.");
    let mut command_lines = io::BufReader::new(io::stdin()).lines();
    while let Some(command_result) = command_lines.next().await {
        let command = command_result?;
        let request = match parse_command(&command) {
            None => continue,
            Some(request) => request,
        };

        utils::send_as_json(&mut to_servier, &request).await?;
        to_servier.flush().await?;
    }
    Ok(())
}

fn parse_command(line: &str) -> Option<FromClient> {
    let (command, rest) = get_next_token(line)?;
    if command == "post" {
        let (group, rest) = get_next_token(rest)?;
        let message = rest.trim_start().to_string();
        return Some(FromClient::Post {
            group_name: Arc::new(group.to_string()),
            message: Arc::new(message),
        });
    } else if command == "join" {
        let (group, rest) = get_next_token(rest)?;
        if !rest.trim_start().is_empty() {
            return None;
        }
        return Some(FromClient::Join {
            group_name: Arc::new(group.to_string()),
        });
    } else {
        eprintln!("Unrecognized command: {:?}", line);
        return None;
    }
}

fn get_next_token(mut input: &str) -> Option<(&str, &str)> {
    input = input.trim_start();

    if input.is_empty() {
        return None;
    }

    match input.find(char::is_whitespace) {
        Some(space) => Some((&input[0..space], &input[space..])),
        None => Some((input, "")),
    }
}
