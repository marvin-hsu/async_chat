use std::sync::Arc;

use async_chat::utils::ChatResult;
use async_std::net::TcpStream;

use crate::group_table::GroupTable;

pub async fn serve(socket:TcpStream,groups:Arc<GroupTable>)->ChatResult<()>{
    Ok(())
}