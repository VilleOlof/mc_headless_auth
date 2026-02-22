use std::thread;

use crossbeam::channel::{Receiver, bounded};

use crate::{
    channel_message::{ChannelMessage, MessageData},
    config::ServerConfig,
    error::MCHAError,
    minecraft,
    token::storage::TokenStorage,
    user::User,
};

pub struct Server {
    pub(crate) r: Receiver<ChannelMessage>,
    pub(crate) storage: TokenStorage,
}

impl Server {
    const CHANNEL_CAPACITY: usize = 1024;

    pub fn start(config: ServerConfig) -> Self {
        let storage = TokenStorage::new(config.token_ttl);

        let (s, r) = bounded::<ChannelMessage>(Self::CHANNEL_CAPACITY);

        let s1 = s.clone();
        thread::spawn(move || {
            minecraft::server::start(config, s1);
        });

        let server = Self { r, storage };

        server.listen_for_joins();

        server
    }

    fn listen_for_joins(&self) {
        let r = self.r.clone();
        let storage = self.storage.clone();
        thread::spawn(move || {
            while let Ok(msg) = r.recv() {
                match msg.data {
                    MessageData::OnJoin { user, token } => {
                        storage.insert(token, user);
                    }
                }
            }
        });
    }

    pub fn verify(&self, token: impl AsRef<str>) -> Result<User, MCHAError> {
        match self.storage.get(&token.as_ref().into()) {
            Some(data) => Ok(data),
            None => Err(MCHAError::NoMatchingToken(token.as_ref().to_string())),
        }
    }
}
