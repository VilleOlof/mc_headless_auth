use bytes::{Buf, BufMut, Bytes};

use crate::minecraft::{
    packet::{ReadPacketData, WritePacketData},
    var_int::VarInt,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PacketString(pub String);

impl PacketString {
    pub fn new(str: impl ToString) -> Self {
        PacketString(str.to_string())
    }
}

impl ReadPacketData for PacketString {
    fn read(data: &mut Bytes) -> PacketString {
        let len = VarInt::read(data).0 as usize;
        let content = data.copy_to_bytes(len);
        PacketString(String::from_utf8(content.to_vec()).unwrap())
    }
}

impl WritePacketData for PacketString {
    fn write(self, data: &mut bytes::BytesMut) {
        let len = VarInt(self.0.len() as i32);
        len.write(data);

        data.put(self.0.as_bytes());
    }
}
