use std::{io::Cursor, time::Duration};

use image::{ImageFormat, RgbaImage};
use serde_json::Value;

use crate::{
    message::{Message, MessageGenerator},
    token::{Token, TokenGenerator},
};

#[derive(Debug, Clone)]
pub struct ServerConfig<T: TokenGenerator = Token, M: MessageGenerator = Message> {
    /// The port on which to bind the minecraft server on
    pub port: u16,
    /// Used to generate the token supplied to the user
    pub token: T,
    /// For how long a token should remain valid until it gets cleared
    pub token_ttl: Duration,
    /// Used to generate the message the user sees when kicked from the server
    pub message: M,
    pub status: StatusConfig,
}

#[derive(Debug, Clone)]
pub struct StatusConfig {
    pub favicon: Option<RgbaImage>,
    pub description: Option<Value>,
    pub legacy_decription: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 25565,
            token: Token,
            token_ttl: Duration::from_secs(5 * 60),
            message: Message,
            status: StatusConfig::default(),
        }
    }
}

pub(crate) const DEFAULT_ICON: &'static [u8] = include_bytes!("../icon.png");
impl Default for StatusConfig {
    fn default() -> Self {
        Self {
            // favicon: None,
            favicon: Some(
                image::load(Cursor::new(DEFAULT_ICON), ImageFormat::Png)
                    .unwrap()
                    .into_rgba8(),
            ),
            // description: Some(Value::String(
            //     "Join to link your minecraft account".to_string(),
            // )),
            description: Some(
                serde_json::to_value(serde_json::json!({
                    "text": "Join to link your minecraft account"
                }))
                .unwrap(),
            ),
            legacy_decription: Some("Join to link your minecraft account".to_string()),
        }
    }
}
