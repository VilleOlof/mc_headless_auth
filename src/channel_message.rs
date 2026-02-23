use rand::{RngExt, rng};

use crate::{ServerError, user::User};

#[derive(Debug)]
pub enum MessageData {
    OnJoin { user: User, token: String },
    ConnectionError(Box<ServerError>),
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
