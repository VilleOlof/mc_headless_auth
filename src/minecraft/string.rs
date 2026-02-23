use bytes::{Buf, Bytes};

use crate::{
    error::TypeError,
    minecraft::{
        packet::{ReadPacketData, WritePacketData},
        var_int::VarInt,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PacketString(pub String);

impl PacketString {
    pub fn new(str: impl ToString) -> Self {
        PacketString(str.to_string())
    }
}

impl ReadPacketData for PacketString {
    fn read(data: &mut Bytes) -> Result<PacketString, TypeError> {
        let len = VarInt::read(data)?.0 as usize;
        let content = data.copy_to_bytes(len);
        Ok(PacketString(String::from_utf8(content.to_vec())?))
    }
}

impl WritePacketData for PacketString {
    fn write(self, data: &mut bytes::BytesMut) {
        VarInt(self.0.len() as i32).write(data);
        data.extend_from_slice(&self.0.into_bytes());
    }
}
