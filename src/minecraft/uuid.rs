use bytes::BytesMut;
use uuid::Uuid;

use crate::minecraft::{packet::WritePacketData, protocol_version, string::PacketString};

// https://github.com/Quozul/PicoLimbo/blob/fcb126585798f78a58d39defde2bac694b132ccb/crates/minecraft_protocol/src/data_types/uuid.rs#L47
pub fn uuid_to_bytes(uuid: Uuid, protocol_version: i32) -> Vec<u8> {
    match protocol_version {
        protocol_version::V1_16.. => uuid.as_bytes().to_vec(),
        protocol_version::V1_7_6.. => {
            let mut data = BytesMut::new();
            PacketString::new(uuid.as_hyphenated()).write(&mut data);
            data.to_vec()
        }
        _ => {
            let mut data = BytesMut::new();
            PacketString::new(uuid.as_simple()).write(&mut data);
            data.to_vec()
        }
    }
}
