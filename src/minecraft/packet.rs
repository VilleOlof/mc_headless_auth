use std::{
    io::{Read, Write},
    net::TcpStream,
};

use bytes::{BufMut, Bytes, BytesMut};
use miniz_oxide::{deflate, inflate};

use crate::{
    error::TypeError,
    minecraft::{
        encrypt::{Aes128CfbDec, Aes128CfbEnc, decrypt_packet, encrypt_packet},
        intents::legacy_ping::compare_init_bytes,
        var_int::VarInt,
    },
};

#[derive(Debug, Clone)]
pub struct Packet {
    pub length: VarInt,
    pub id: VarInt,
    pub data: Bytes,
}

impl Packet {
    pub(crate) const PACKET_LIMIT: usize = 2097151;

    pub fn new(id: i32, data: Bytes) -> Self {
        Self {
            length: VarInt(data.len() as i32),
            id: VarInt(id),
            data,
        }
    }

    pub fn read_init(stream: &mut TcpStream) -> Result<InitPacket, TypeError> {
        // read 3 first bytes and see if they align with any pre 1.7 list pings
        // if not, we can pass the first 3 bytes and the stream to the varint processing
        let mut packet_ident = [0u8; 3];
        stream
            .read(&mut packet_ident)
            .map_err(|e| TypeError::ReadError(e))?;

        // check the first 3 bytes against some patterns
        // and if any of these match, its a legacy that we handle differently
        // since their format is WAY different than the modern packet format
        if let Some(legacy) = compare_init_bytes(packet_ident) {
            return Ok(legacy);
        }

        let mut packet_ident = packet_ident.to_vec();
        let len = VarInt::read_via_stream(stream, &mut packet_ident)?;
        let buf_len = len.0 as usize - packet_ident.len();

        if len.0 > Self::PACKET_LIMIT as i32 {
            return Err(TypeError::PacketSizeExceedsLimit(len.0));
        }

        let mut data = vec![0u8; buf_len];

        stream
            .read_exact(&mut data)
            .map_err(|e| TypeError::ReadError(e))?;
        // if theres any bytes left in packet_ident
        // we put that into data and negative the len
        packet_ident.append(&mut data);
        let data = packet_ident;

        let mut data = Bytes::from_owner(data);
        let id = VarInt::read(&mut data)?;

        Ok(InitPacket::V1_7Above(Self {
            length: len,
            id,
            data,
        }))
    }

    pub fn from_stream(stream: &mut TcpStream, expected_id: i32) -> Result<Self, TypeError> {
        let len = VarInt::read_via_stream(stream, &mut Vec::new())?;

        if len.0 > Self::PACKET_LIMIT as i32 {
            return Err(TypeError::PacketSizeExceedsLimit(len.0));
        }

        let mut data = vec![0u8; len.0 as usize];
        stream.read_exact(&mut data).unwrap();

        let mut data = Bytes::from_owner(data);
        let id = VarInt::read(&mut data)?;

        if id.0 != expected_id {
            return Err(TypeError::UnexpectedPacketId(expected_id, id.0));
        }

        Ok(Self {
            length: len,
            id,
            data,
        })
    }

    #[must_use]
    pub fn write_stream(self, stream: &mut TcpStream) -> Result<(), TypeError> {
        let mut data = BytesMut::new();
        self.write(&mut data);

        stream
            .write_all(&data)
            .map_err(|e| TypeError::WriteError(e))?;

        Ok(())
    }

    #[must_use]
    pub fn write_encrypted_stream(
        self,
        stream: &mut TcpStream,
        enc: &mut Aes128CfbEnc,
    ) -> Result<(), TypeError> {
        let mut data = BytesMut::new();
        self.write(&mut data);
        let mut data = data.to_vec();
        encrypt_packet(enc, &mut data)?;

        stream
            .write_all(&data)
            .map_err(|e| TypeError::WriteError(e))?;

        Ok(())
    }

    #[must_use]
    pub fn write_compressed_encrypted_stream(
        self,
        stream: &mut TcpStream,
        enc: &mut Aes128CfbEnc,
    ) -> Result<(), TypeError> {
        let mut packet_data = BytesMut::new();
        self.id.write(&mut packet_data);
        packet_data.extend_from_slice(&self.data);
        let comp_packet = &deflate::compress_to_vec_zlib(&packet_data[..], 4);

        let uncomp_len = self.id.byte_len() + self.data.len();

        let data_len = VarInt(uncomp_len as i32).byte_len() + comp_packet.len();

        let mut packet = BytesMut::new();
        VarInt(data_len as i32).write(&mut packet);
        VarInt(uncomp_len as i32).write(&mut packet);

        packet.extend_from_slice(comp_packet);

        let mut packet = packet.to_vec();
        encrypt_packet(enc, &mut packet)?;
        stream
            .write_all(&packet)
            .map_err(|e| TypeError::WriteError(e))?;

        Ok(())
    }

    pub fn from_compressed_encrypted_stream(
        stream: &mut TcpStream,
        dec: &mut Aes128CfbDec,
        expected_id: i32,
    ) -> Result<Self, TypeError> {
        let len = VarInt::read_via_encrypted_stream(stream, dec)?;
        let mut data = vec![0u8; len.0 as usize];
        stream.read_exact(&mut data).unwrap();
        decrypt_packet(dec, &mut data)?;
        let mut data = Bytes::from_owner(data);

        let data_len = VarInt::read(&mut data)?;
        // id + data is compressed
        let mut data = Bytes::from_owner(
            inflate::decompress_to_vec_zlib(&data).map_err(|e| TypeError::DecompressError(e))?,
        );
        let id = VarInt::read(&mut data)?;

        if id.0 != expected_id {
            return Err(TypeError::UnexpectedPacketId(expected_id, id.0));
        }

        Ok(Self {
            length: data_len,
            id,
            data,
        })
    }
}

impl WritePacketData for Packet {
    fn write(mut self, data: &mut BytesMut) {
        // len should include id
        let mut len = 0;
        len += self.id.byte_len();
        len += self.data.len();
        self.length = VarInt(len as i32);

        self.length.write(data);
        self.id.write(data);
        data.put(self.data);
    }
}

pub trait ReadPacketData {
    fn read(data: &mut Bytes) -> Result<Self, TypeError>
    where
        Self: Sized;
}

pub trait WritePacketData {
    fn write(self, data: &mut BytesMut);

    fn byte_len(self) -> usize
    where
        Self: Sized,
    {
        let mut data = BytesMut::new();
        self.write(&mut data);
        data.len()
    }
}

impl WritePacketData for u8 {
    fn write(self, data: &mut BytesMut) {
        data.put_u8(self);
    }
}

#[derive(Debug, Clone)]
pub enum InitPacket {
    V1_6,
    V1_4To1_5,
    Vbeta1_8To1_3,
    V1_7Above(Packet),
}
