pub mod array;
pub mod auth;
pub mod encrypt;
pub mod handshake;
pub mod hash;
pub mod intents;
pub mod login_start;
pub mod optional;
pub mod packet;
pub mod server;
pub mod string;
pub mod uuid;
pub mod var_int;

/// Special Protocol Versions that are used in special cases
pub mod protocol_version {
    pub const V1_21_2: i32 = 768;
    pub const V1_20_5: i32 = 766;
    pub const V1_16: i32 = 735;
    pub const V1_7_6: i32 = 5;

    pub const MIN_SUPPORTED_PROTOCOL: i32 = V1_21_2;
}

mod packets {
    use bytes::{BufMut, BytesMut};
    use serde_json::json;
    use simdnbt::owned::NbtTag;

    use crate::minecraft::{
        array::Array,
        auth::GameProfileProps,
        packet::{Packet, WritePacketData},
        string::PacketString,
        var_int::VarInt,
    };

    pub fn encryption_request(
        server_id: &str,
        public_key: Vec<u8>,
        verify_token: Vec<u8>,
        should_authenticate: bool,
    ) -> Packet {
        let mut data = BytesMut::new();

        PacketString::new(server_id).write(&mut data);
        Array::new(public_key).write(&mut data);
        Array::new(verify_token).write(&mut data);
        data.put_u8(should_authenticate as u8);

        Packet::new(0x01, data.into())
    }

    pub fn set_compression(threshold: i32) -> Packet {
        let mut data = BytesMut::new();

        VarInt(threshold).write(&mut data);

        Packet::new(0x03, data.into())
    }

    pub fn login_success(
        uuid: Vec<u8>,
        username: &str,
        _properties: Vec<GameProfileProps>,
    ) -> Packet {
        let mut data = BytesMut::new();

        data.extend_from_slice(&uuid);
        PacketString::new(username).write(&mut data);

        VarInt(_properties.len() as i32).write(&mut data);
        for prop in _properties {
            PacketString::new(prop.name).write(&mut data);
            PacketString::new(prop.value).write(&mut data);
            prop.signature
                .map(|s| PacketString::new(s))
                .write(&mut data);
        }

        Packet::new(0x02, data.into())
    }

    pub fn status_response(status: String) -> Packet {
        let mut data = BytesMut::new();

        PacketString::new(status).write(&mut data);

        Packet::new(0x00, data.into())
    }

    pub fn pong_response(timestamp: i64) -> Packet {
        let mut data = BytesMut::new();

        data.put_i64(timestamp);

        Packet::new(0x01, data.into())
    }

    pub fn disconnect_configuration(text_component: NbtTag) -> Packet {
        let mut data = BytesMut::new();

        let mut msg = Vec::new();
        text_component.write(&mut msg);
        data.extend_from_slice(&msg);

        Packet::new(0x02, data.into())
    }

    pub fn disconnect_login(reason: &str) -> Packet {
        let mut data = BytesMut::new();

        let reason = json!({
            "text": reason
        });
        let json = serde_json::to_string(&reason).unwrap();

        PacketString::new(json).write(&mut data);

        Packet::new(0x00, data.into())
    }
}
