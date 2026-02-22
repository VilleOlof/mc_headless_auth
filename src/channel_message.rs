use rand::{RngExt, rng};

use crate::user::User;

#[derive(Debug, Clone)]
pub enum MessageData {
    OnJoin { user: User, token: String },
}

#[derive(Debug, Clone)]
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
