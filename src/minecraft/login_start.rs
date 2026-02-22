use bytes::Buf;
use uuid::Uuid;

use crate::minecraft::{packet::ReadPacketData, string::PacketString};

#[derive(Debug, Clone)]
pub struct LoginStart {
    pub name: PacketString,
    pub uuid: Option<Uuid>,
}

impl ReadPacketData for LoginStart {
    fn read(data: &mut bytes::Bytes) -> Self {
        let name = PacketString::read(data);
        let uuid = if data.len() > 0 {
            Some(Uuid::from_u128(data.get_u128()))
        } else {
            None
        };

        return Self { name, uuid };
    }
}
