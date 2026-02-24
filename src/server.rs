use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crossbeam::channel::{Sender, bounded};

use crate::{
    MCHAError, ServerError,
    broadcast::Broadcast,
    channel_message::{ChannelMessage, MessageData},
    config::ServerConfig,
    minecraft,
    player::Player,
    token::storage::TokenStorage,
};

/// The consumer end of the Minecraft server  
///
/// Holds an internal connection to the thread running the Minecraft server.  
///
/// Use this to [`start`](Server::start) it and get events from it.  
#[derive(Clone)]
pub struct Server {
    pub(crate) broadcast: Broadcast,
    pub(crate) storage: TokenStorage,
    pub(crate) server_signal: Sender<ChannelMessage>,
    pub(crate) server_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Drop for Server {
    fn drop(&mut self) {
        // make sure the server isnt active at all after the server exits
        // i dont exactly know what happens to the event threads from on_error or on_join when the server drops
        // as they are on different threads, but they do return a joinhandle so the consumer could fix it
        let _ = self.shutdown();
    }
}

impl Server {
    const CHANNEL_CAPACITY: usize = 1024;

    /// Starts a Minecraft Server in a background thread *(which can be shutdown with [`Server::shutdown`])*  
    ///
    /// This server disconnects the player as soon as it as authenticated them against Mojangs servers.  
    ///
    /// And then stores their `username` + `uuid` with a generated [`Token`](crate::TokenGenerator).  
    ///
    /// Using this token with [`Server::verify`], the player can be retreived later.  
    pub fn start(config: ServerConfig) -> Self {
        let storage = TokenStorage::new(config.token_ttl);
        let broadcast = Broadcast::new();

        let _broadcast = broadcast.clone();

        let (s_s, s_r) = bounded(Self::CHANNEL_CAPACITY);
        let s_t = thread::spawn(move || {
            minecraft::server::start(config, _broadcast, s_r);
        });

        let server = Self {
            broadcast,
            storage,
            server_signal: s_s,
            server_handle: Arc::new(Mutex::new(Some(s_t))),
        };

        let _storage = server.storage.clone();
        server.on_join(move |user, token| {
            _storage.insert(token.clone(), user.clone());
        });

        server
    }

    /// Returns a associated [`Player`] if the given token is a valid one.  
    ///
    /// ## Example
    /// ```no_run
    /// let server = Server::start(ServerConfig::default());
    ///
    /// let token = String::from("MJMMJSLXHG");
    /// let player = server.verify(&token);
    /// assert!(player.is_some());
    /// ```
    pub fn verify(&self, token: impl AsRef<str>) -> Option<Player> {
        self.storage.get(&token.as_ref().into())
    }

    /// A function to execute if a player connection in the server thread errors out.  
    ///
    /// Can be useful to log and monitor the Minecraft server.
    ///
    /// ## Example
    /// ```no_run
    /// let server = Server::start(ServerConfig::default());
    /// let _ = server.on_error(|e| {
    ///     eprintln!("{e:?}");
    /// });
    /// ```
    pub fn on_error(
        &self,
        handler: impl Fn(&ServerError) + Send + Sync + 'static,
    ) -> JoinHandle<()> {
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
        })
    }

    /// A function to execute upon every successful join and disconnect.  
    ///
    /// ## Example
    /// ```no_run
    /// let server = Server::start(ServerConfig::default());
    /// let _ = server.on_join(|player, token| {
    ///     println!("{} just joined and got {}", player.username, token);
    /// });
    /// ```
    pub fn on_join(
        &self,
        handler: impl Fn(&Player, &String) + Send + Sync + 'static,
    ) -> JoinHandle<()> {
        let b = self.broadcast.clone();
        thread::spawn(move || {
            let r = b.sub(Self::CHANNEL_CAPACITY);
            while let Ok(msg) = r.recv() {
                match &msg.data {
                    MessageData::OnJoin { player, token } => {
                        handler(player, token);
                    }
                    _ => (),
                }
            }
        })
    }

    /// Sends a signal for the server to terminate itself and stop accepting connections.  
    ///
    /// This function also guarantees that the thread holding the server has returned fully.  
    ///
    /// ## Example
    /// ```no_run
    /// // Turn off the server after 5 seconds.  
    /// let server = Server::start(ServerConfig::default());
    /// sleep(Duration::from_secs(5));
    /// server.shutdown().unwrap();
    /// ```
    pub fn shutdown(&self) -> Result<(), MCHAError> {
        let handle = self.server_handle.lock().unwrap().take();
        if handle.is_none() {
            return Err(MCHAError::NoServerRunning);
        }

        self.server_signal
            .send(ChannelMessage::new(MessageData::CloseServer))?;

        if let Some(h) = handle {
            h.join().map_err(|e| MCHAError::ThreadError(e))?;
        }

        Ok(())
    }
}
