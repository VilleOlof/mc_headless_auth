use bytes::Buf;
use uuid::Uuid;

use crate::{
    error::TypeError,
    minecraft::{packet::ReadPacketData, string::PacketString},
};

#[derive(Debug, Clone)]
pub struct LoginStart {
    pub name: PacketString,
    pub uuid: Option<Uuid>,
}

impl ReadPacketData for LoginStart {
    fn read(data: &mut bytes::Bytes) -> Result<Self, TypeError> {
        let name = PacketString::read(data)?;
        let uuid = if data.len() > 0 {
            Some(Uuid::from_u128(data.get_u128()))
        } else {
            None
        };

        Ok(Self { name, uuid })
    }
}
