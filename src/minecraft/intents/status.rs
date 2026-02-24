use std::{io::Cursor, net::TcpStream};

use bytes::Buf;
use constcat::concat;
use image::{ImageFormat, RgbaImage};
use serde::Serialize;
use serde_json::Value;

use crate::{
    ServerError,
    config::{MIN_SUPPORTED_VERSION, StatusConfig},
    message::MessageGenerator,
    minecraft::{
        handshake::Handshake, packet::Packet, packets, protocol_version, server::ConnectionState,
    },
    token::TokenGenerator,
};

pub fn advance<T: TokenGenerator, M: MessageGenerator>(
    stream: &mut TcpStream,
    _state: ConnectionState<T, M>,
    handshake: Handshake,
    config: StatusConfig,
) -> Result<(), ServerError> {
    let _ = Packet::from_stream(stream, 0x00)?;

    let status = get_status(handshake.protocol_version.0, config)?;
    packets::status_response(status).write_stream(stream)?;

    let mut ping = Packet::from_stream(stream, 0x01)?;
    let timestamp = ping.data.get_i64();

    packets::pong_response(timestamp).write_stream(stream)?;

    Ok(())
}

fn get_status(protocol: i32, config: StatusConfig) -> Result<String, ServerError> {
    let status = StatusResponse {
        version: Version {
            name: concat!(MIN_SUPPORTED_VERSION, "+").to_string(),
            protocol,
        },
        players: Some(Players {
            max: 0,
            online: 0,
            sample: Vec::default(),
        }),
        description: if protocol < protocol_version::MIN_SUPPORTED_PROTOCOL {
            Some(Value::String(config.legacy_decription.unwrap_or_default()))
        } else {
            config.description
        },
        favicon: encode_favicon(config.favicon)?,
        enforces_secure_chat: false,
    };

    Ok(serde_json::to_string(&status)?)
}

fn encode_favicon(favicon: Option<RgbaImage>) -> Result<Option<String>, ServerError> {
    use base64::prelude::*;

    let favicon = match favicon {
        Some(f) => f,
        None => return Ok(None),
    };

    if favicon.width() > 64 || favicon.height() > 64 {
        return Ok(None);
    }

    let mut img = Cursor::new(Vec::new());
    favicon.write_to(&mut img, ImageFormat::Png)?;

    Ok(Some(format!(
        "data:image/png;base64,{}",
        BASE64_STANDARD.encode(img.get_ref())
    )))
}

#[derive(Debug, Clone, Serialize)]
struct StatusResponse {
    version: Version,
    #[serde(skip_serializing_if = "Option::is_none")]
    players: Option<Players>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    favicon: Option<String>,
    #[serde(rename = "enforcesSecureChat")]
    enforces_secure_chat: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Version {
    name: String,
    protocol: i32,
}

#[derive(Debug, Clone, Serialize)]
struct Players {
    max: i32,
    online: i32,
    sample: Vec<Player>,
}

#[derive(Debug, Clone, Serialize)]
struct Player {
    name: String,
    id: String,
}
