use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub enum MsgType {
    MsgChat,
    MsgJoin,
    MsgLeave,
}

#[derive(Deserialize)]
pub struct ChatMessage {
    pub mtype: MsgType,
    pub msg: String,
}

#[derive(Serialize)]
pub struct UsrMessage {
    pub from_user: String,
    pub msg: String,
}
