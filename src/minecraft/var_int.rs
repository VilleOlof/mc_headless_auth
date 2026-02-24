use std::{io::Read, net::TcpStream};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::{
    error::TypeError,
    minecraft::{
        encrypt::{Aes128CfbDec, decrypt_packet},
        packet::{ReadPacketData, WritePacketData},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarInt(pub i32);
impl VarInt {
    const SEGMENT_BITS: u8 = 0x7F;
    const CONTINUE_BIT: u8 = 0x80;

    // TODO: make this all more generic ass
    pub fn read_via_stream(
        stream: &mut TcpStream,
        pre_bytes: &mut Vec<u8>,
    ) -> Result<Self, TypeError> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        while position < 32 {
            // pre-bytes are some bytes from the packet we have already read
            // incase they were a legacy packet but if normal packet we just use those bytes
            // towards the varint len instead
            let byte = if !pre_bytes.is_empty() {
                pre_bytes.remove(0)
            } else {
                let mut buf = [0u8; 1];
                stream
                    .read_exact(&mut buf)
                    .map_err(|e| TypeError::ReadError(e))?;
                buf[0]
            };

            value |= ((byte & Self::SEGMENT_BITS) as i32) << position;

            if (byte & Self::CONTINUE_BIT) == 0 {
                return Ok(VarInt(value));
            }

            position += 7;
        }

        Err(TypeError::OversizedVarInt(value))
    }

    pub fn read_via_encrypted_stream(
        stream: &mut TcpStream,
        dec: &mut Aes128CfbDec,
    ) -> Result<Self, TypeError> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        while position < 32 {
            let mut buf = [0u8; 1];
            stream
                .read_exact(&mut buf)
                .map_err(|e| TypeError::ReadError(e))?;
            decrypt_packet(dec, &mut buf)?;
            let byte = buf[0];

            value |= ((byte & Self::SEGMENT_BITS) as i32) << position;

            if (byte & Self::CONTINUE_BIT) == 0 {
                return Ok(VarInt(value));
            }

            position += 7;
        }

        Err(TypeError::OversizedVarInt(value))
    }
}

impl ReadPacketData for VarInt {
    fn read(data: &mut Bytes) -> Result<Self, TypeError> {
        let mut value: i32 = 0;
        let mut position: i32 = 0;

        while position < 32 {
            let byte = data.get_u8();
            value |= ((byte & Self::SEGMENT_BITS) as i32) << position;

            if (byte & Self::CONTINUE_BIT) == 0 {
                return Ok(VarInt(value));
            }

            position += 7;
        }

        Err(TypeError::OversizedVarInt(value))
    }
}

impl WritePacketData for VarInt {
    fn write(self, data: &mut BytesMut) {
        let mut value = self.0 as u32;

        loop {
            if (value & !(Self::SEGMENT_BITS as u32)) == 0 {
                data.put_u8(value as u8);
                return;
            } else {
                data.put_u8(
                    ((value & (Self::SEGMENT_BITS as u32)) | (Self::CONTINUE_BIT as u32)) as u8,
                );
                value >>= 7;
            }
        }
    }
}
