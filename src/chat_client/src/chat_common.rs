use serde::{Deserialize, Serialize};

#[derive(Serialize, PartialEq, Clone)]
pub enum MsgType {
    MsgChat,
    MsgJoin,
    MsgLeave,
}

#[derive(Serialize)]
pub struct ChatMessage {
    pub mtype: MsgType,
    pub msg: String,
}

#[derive(Deserialize)]
pub struct UsrMessage {
    pub from_user: String,
    pub msg: String,
}
