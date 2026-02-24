use rand::{RngExt, rng};

use crate::{ServerError, player::Player};

#[derive(Debug)]
pub enum MessageData {
    OnJoin { player: Player, token: String },
    ConnectionError(Box<ServerError>),
    CloseServer,
}

#[derive(Debug)]
pub struct ChannelMessage {
    pub _id: i64,
    pub data: MessageData,
}

impl ChannelMessage {
    pub fn new(data: MessageData) -> Self {
        let id = rng().random_range(i64::MIN..=i64::MAX);
        Self { _id: id, data }
    }
}
