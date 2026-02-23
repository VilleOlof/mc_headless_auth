use bytes::Buf;

use crate::{
    error::TypeError,
    minecraft::{
        packet::{ReadPacketData, WritePacketData},
        var_int::VarInt,
    },
};

#[derive(Debug, Clone)]
pub struct Array<T: WritePacketData> {
    len: VarInt,
    data: Vec<T>,
}

impl<T: WritePacketData> Array<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self {
            len: VarInt(data.len() as i32),
            data,
        }
    }

    pub fn as_ref(&self) -> &Vec<T> {
        &self.data
    }
}

impl<T: WritePacketData> WritePacketData for Array<T> {
    fn write(self, data: &mut bytes::BytesMut) {
        self.len.write(data);
        for item in self.data {
            item.write(data);
        }
    }
}

impl ReadPacketData for Array<u8> {
    fn read(data: &mut bytes::Bytes) -> Result<Self, TypeError> {
        let len = VarInt::read(data)?;

        let v_data = data.copy_to_bytes(len.0 as usize).to_vec();

        Ok(Array { len, data: v_data })
    }
}
