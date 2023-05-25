use std::sync::Arc;

use async_chat::{
    utils::{self, ChatResult},
    FromClient, FromServer,
};
use async_std::{
    io::{BufReader, WriteExt},
    net::TcpStream,
    stream::StreamExt,
    sync::Mutex,
};

use crate::group_table::GroupTable;

pub struct OutBound(Mutex<TcpStream>);

impl OutBound {
    pub fn new(to_client: TcpStream) -> Self {
        Self(Mutex::new(to_client))
    }

    pub async fn send(&self, packet: FromServer) -> ChatResult<()> {
        let mut guard = self.0.lock().await;
        utils::send_as_json(&mut *guard, &packet).await?;
        guard.flush().await?;
        Ok(())
    }
}

pub async fn serve(socket: TcpStream, groups: Arc<GroupTable>) -> ChatResult<()> {
    let outbound = Arc::new(OutBound::new(socket.clone()));

    let buffered = BufReader::new(socket);
    let mut from_client = utils::receive_as_json(buffered);
    while let Some(request_result) = from_client.next().await {
        let request = request_result?;

        let result = match request {
            FromClient::Join { group_name } => {
                let group = groups.get_or_create(group_name);
                group.join(outbound.clone());
                Ok(())
            }
            FromClient::Post {
                group_name,
                message,
            } => match groups.get(&group_name) {
                Some(group) => {
                    group.post(message);
                    Ok(())
                }
                None => Err(format!("Group '{}' does not exist", group_name)),
            },
        };

        if let Err(e) = result {
            let report = FromServer::Error(e);
            outbound.send(report).await?;
        }
    }
    Ok(())
}
