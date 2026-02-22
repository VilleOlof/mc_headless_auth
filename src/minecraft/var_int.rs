use std::{io::Read, net::TcpStream};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::minecraft::{
    encrypt::{Aes128CfbDec, decrypt_packet},
    packet::{ReadPacketData, WritePacketData},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarInt(pub i32);
impl VarInt {
    const SEGMENT_BITS: u8 = 0x7F;
    const CONTINUE_BIT: u8 = 0x80;

    // TODO make VarInt via different context's more generic
    pub fn read_via_stream(stream: &mut TcpStream) -> Self {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        while position < 32 {
            let mut buf = [0u8; 1];
            stream.read_exact(&mut buf).unwrap();
            let byte = buf[0];

            value |= ((byte & Self::SEGMENT_BITS) as i32) << position;

            if (byte & Self::CONTINUE_BIT) == 0 {
                return VarInt(value);
            }

            position += 7;
        }

        panic!("overzied varint")
    }

    pub fn read_via_encrypted_stream(stream: &mut TcpStream, dec: &mut Aes128CfbDec) -> Self {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        while position < 32 {
            let mut buf = [0u8; 1];
            stream.read_exact(&mut buf).unwrap();
            decrypt_packet(dec, &mut buf);
            let byte = buf[0];

            value |= ((byte & Self::SEGMENT_BITS) as i32) << position;

            if (byte & Self::CONTINUE_BIT) == 0 {
                return VarInt(value);
            }

            position += 7;
        }

        panic!("overzied varint")
    }
}

impl ReadPacketData for VarInt {
    fn read(data: &mut Bytes) -> Self {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        while position < 32 {
            let byte = data.get_u8();
            value |= ((byte & Self::SEGMENT_BITS) as i32) << position;

            if (byte & Self::CONTINUE_BIT) == 0 {
                return VarInt(value);
            }

            position += 7;
        }

        panic!("overized varint");
    }
}

impl WritePacketData for VarInt {
    fn write(mut self, data: &mut BytesMut) {
        while (self.0 & !Self::SEGMENT_BITS as i32) != 0 {
            data.put_u8((self.0 & Self::SEGMENT_BITS as i32 | Self::CONTINUE_BIT as i32) as u8);
            self.0 >>= 7;
        }
        data.put_u8(self.0 as u8);
    }
}
