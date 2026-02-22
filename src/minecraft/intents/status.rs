use std::{
    io::{BufWriter, Cursor},
    net::TcpStream,
};

use bytes::{Buf, BufMut, BytesMut};
use image::{ImageFormat, RgbaImage};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
    config::StatusConfig,
    message::MessageGenerator,
    minecraft::{
        handshake::Handshake,
        packet::{self, Packet, WritePacketData},
        server::ConnectionState,
        string::PacketString,
    },
    token::TokenGenerator,
};

pub fn advance<T: TokenGenerator, M: MessageGenerator>(
    stream: &mut TcpStream,
    state: ConnectionState<T, M>,
    handshake: Handshake,
    config: StatusConfig,
) {
    // 3. Status Request
    let status_request = Packet::from_stream(stream);
    if status_request.id.0 != 0x00 {
        panic!("mismatched status_request packet id");
    }

    // 4. Status Response
    let status = get_status(handshake.protocol_version.0, config);
    println!("{status:?}");
    let packet_string = PacketString::new(status);
    let mut data = BytesMut::new();
    packet_string.write(&mut data);
    Packet::new(0x00, data.into()).write_stream(stream);
    println!("sent status response");

    // 5. Ping Request
    let mut ping = Packet::from_stream(stream);
    if ping.id.0 != 0x01 {
        panic!("mismatched ping_request packet id");
    }
    let timestamp = ping.data.get_i64();
    println!("got ping request: {timestamp}");

    // 6. Pong Response
    let mut data = BytesMut::new();
    data.put_i64(timestamp);
    Packet::new(0x01, data.into()).write_stream(stream);
    println!("sent pong response")
}

fn get_status(protocol: i32, config: StatusConfig) -> String {
    let status = StatusResponse {
        version: Version {
            name: "1.7+".to_string(),
            protocol,
        },
        players: Players {
            max: 0,
            online: 0,
            sample: Vec::default(),
        },
        description: config.description,
        favicon: encode_favicon(config.favicon),
        enforces_secure_chat: false,
    };

    serde_json::to_string_pretty(&status).unwrap()
}

fn encode_favicon(favicon: Option<RgbaImage>) -> Option<String> {
    use base64::prelude::*;

    let favicon = match favicon {
        Some(f) => f,
        None => return None,
    };

    let mut img = Cursor::new(Vec::new());
    favicon.write_to(&mut img, ImageFormat::Png).unwrap();

    Some(format!(
        "data:image/png;base64,{}",
        BASE64_STANDARD.encode(img.get_ref())
    ))
}

#[derive(Debug, Clone, Serialize)]
struct StatusResponse {
    version: Version,
    players: Players,
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
