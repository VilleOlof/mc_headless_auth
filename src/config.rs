use std::{io::Cursor, time::Duration};

use constcat::concat;
use image::{ImageFormat, RgbaImage};
use serde_json::Value;

use crate::{
    message::{Message, MessageGenerator},
    token::{Token, TokenGenerator},
};

pub(crate) const MIN_SUPPORTED_VERSION: &'static str = "1.21.2";
pub(crate) const DEFAULT_DESC: &'static str = "Join to link your minecraft account";
pub(crate) const DEFAULT_LEGACY_DESC: &'static str =
    concat!("Use a client newer than ", MIN_SUPPORTED_VERSION, "+");
pub(crate) const DEFAULT_ICON: &'static [u8] = include_bytes!("../icon.png");

/// Config for the Minecraft Server
#[derive(Debug, Clone)]
pub struct ServerConfig<T: TokenGenerator = Token, M: MessageGenerator = Message> {
    /// The port on which to bind the minecraft server on
    ///
    /// Defaults to `25565` which is the default Minecraft port.  
    pub port: u16,
    /// Used to generate the token supplied to the user
    ///
    /// Defaults to [`Token`], which will be a 10 uppercase letter token.  
    pub token: T,
    /// For how long a token should remain valid until it gets cleared
    ///
    /// This ttl is also used for the interval in which tokens ttl are checked.  
    ///
    /// Defaults to `5 minutes`
    pub token_ttl: Duration,
    /// Used to generate the message the user sees when kicked from the server
    ///
    /// Defaults to [`Message`], which will display the token in a green small font,  
    /// and then have a little notice about using the token to link your account.  
    pub message: M,
    /// Config for status packets, values for server favicon, description etc.  
    pub status: StatusConfig,
}

/// Config for status packets
#[derive(Debug, Clone)]
pub struct StatusConfig {
    /// A 64x64 PNG image to be used as the servers favicon.  
    ///
    /// ## Default Favicon
    /// ![icon.png](https://bimply.lifelike.dev/d/ZyRdaH55Au)
    pub favicon: Option<RgbaImage>,
    /// A description for the server, must be a valid [text component](https://minecraft.wiki/w/Text_component_format)
    pub description: Option<Value>,
    /// Description used for legacy pings.  
    ///
    /// This should be used as a warning message, since servers before 1.7 can't join at all.  
    ///
    /// But we still respond to server list pings to indicate that the server exists.
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

impl Default for StatusConfig {
    fn default() -> Self {
        Self {
            favicon: Some(
                image::load(Cursor::new(DEFAULT_ICON), ImageFormat::Png)
                    .unwrap()
                    .into_rgba8(),
            ),
            description: Some(
                serde_json::to_value(serde_json::json!({
                    "text": DEFAULT_DESC
                }))
                .unwrap(),
            ),
            legacy_decription: Some(DEFAULT_LEGACY_DESC.to_string()),
        }
    }
}
