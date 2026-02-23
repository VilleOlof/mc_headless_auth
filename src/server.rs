use std::thread;

use crate::{
    ServerError, broadcast::Broadcast, channel_message::MessageData, config::ServerConfig,
    minecraft, token::storage::TokenStorage, user::User,
};

/// The consumer end of the Minecraft server  
///
/// Holds an internal connection to the thread running the Minecraft server.  
///
/// Use this to start it and get events from it.  
#[derive(Clone)]
pub struct Server {
    pub(crate) broadcast: Broadcast,
    pub(crate) storage: TokenStorage,
}

impl Server {
    const CHANNEL_CAPACITY: usize = 1024;

    pub fn start(config: ServerConfig) -> Self {
        let storage = TokenStorage::new(config.token_ttl);
        let broadcast = Broadcast::new();

        let _broadcast = broadcast.clone();
        thread::spawn(move || {
            minecraft::server::start(config, _broadcast);
        });

        let server = Self { broadcast, storage };

        let _storage = server.storage.clone();
        server.on_join(move |user, token| {
            _storage.insert(token.clone(), user.clone());
        });

        server
    }

    pub fn verify(&self, token: impl AsRef<str>) -> Option<User> {
        self.storage.get(&token.as_ref().into())
    }

    pub fn on_error(&self, handler: impl Fn(&ServerError) + Send + Sync + 'static) {
        let b = self.broadcast.clone();
        thread::spawn(move || {
            let r = b.sub(Self::CHANNEL_CAPACITY);
            while let Ok(msg) = r.recv() {
                match &msg.data {
                    MessageData::ConnectionError(e) => {
                        handler(e.as_ref());
                    }
                    _ => (),
                }
            }
        });
    }

    pub fn on_join(&self, handler: impl Fn(&User, &String) + Send + Sync + 'static) {
        let b = self.broadcast.clone();
        thread::spawn(move || {
            let r = b.sub(Self::CHANNEL_CAPACITY);
            while let Ok(msg) = r.recv() {
                match &msg.data {
                    MessageData::OnJoin { user, token } => {
                        handler(user, token);
                    }
                    _ => (),
                }
            }
        });
    }
}
