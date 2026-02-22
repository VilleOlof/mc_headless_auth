use std::{
    io::{Read, Write},
    net::TcpStream,
};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::minecraft::{
    encrypt::{Aes128CfbDec, Aes128CfbEnc, decrypt_packet, encrypt_packet},
    var_int::VarInt,
};

#[derive(Debug, Clone)]
pub struct Packet {
    pub length: VarInt,
    pub id: VarInt,
    pub data: Bytes,
}

impl Packet {
    const PACKET_LIMIT: usize = 2097151;

    pub fn new(id: i32, data: Bytes) -> Self {
        Self {
            length: VarInt(data.len() as i32),
            id: VarInt(id),
            data,
        }
    }

    pub fn from_stream(stream: &mut TcpStream) -> Self {
        let len = VarInt::read_via_stream(stream);
        let mut data = vec![0u8; len.0 as usize];
        stream.read_exact(&mut data).unwrap();
        let mut data = Bytes::from_owner(data);
        let id = VarInt::read(&mut data);
        println!("len:{}, id:{}, {:?}", len.0, id.0, data.to_vec());

        // TODO, kinda have to check these bytes at the same time as getting them as varInts for the len
        // depends on the version which we dont know, so like if the first byte is an 0xFE followed by these other bytes
        // just go into legacy ping (shouldnt go into legacy if the len just happen to match 0xFE)
        // then bail into legacy but if its a valid varint and doesnt match these bytes then continue as normal
        // legacy always starts with 0xFE
        if data.len() >= 3 && data[0] == 0xFE {
            // legacy ping, so kinda custom format a packet that handles it later
            // look at legacy_ping::match_id_to_version for the version matching
            // but no packet is negative so we play with that
            let mut custom_packet = Self {
                length: VarInt(-1),
                id: VarInt(-125),
                data: Bytes::new(),
            };

            if data[0..3] == [0xFE, 0x01, 0xFA] {
                custom_packet.id.0 = -127;
                return custom_packet;
            }
            if data[0..2] == [0xFE, 0x01] {
                custom_packet.id.0 = -126;
                return custom_packet;
            }

            return custom_packet;
        }

        Self {
            length: len,
            id,
            data,
        }
    }

    pub fn from_encrypted_stream(stream: &mut TcpStream, dec: &mut Aes128CfbDec) -> Self {
        println!("encypting stream");
        // TODO: 1.16 / 1.8 / earlier versions than the most recents just shit themself here
        let len = VarInt::read_via_encrypted_stream(stream, dec);
        println!("packet len: {len:?}");
        let mut data = vec![0u8; len.0 as usize];
        stream.read_exact(&mut data).unwrap();
        decrypt_packet(dec, &mut data);
        let mut data = Bytes::from_owner(data);
        let id = VarInt::read(&mut data);

        Self {
            length: len,
            id,
            data,
        }
    }

    pub fn write_stream(self, stream: &mut TcpStream) {
        let mut data = BytesMut::new();
        self.write(&mut data);

        stream.write_all(&data).unwrap();
    }

    pub fn write_encrypted_stream(self, stream: &mut TcpStream, enc: &mut Aes128CfbEnc) {
        let mut data = BytesMut::new();
        self.write(&mut data);
        let mut data = data.to_vec();
        encrypt_packet(enc, &mut data);

        stream.write_all(&data).unwrap();
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
    fn read(data: &mut Bytes) -> Self;
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

impl WritePacketData for u128 {
    fn write(self, data: &mut BytesMut) {
        data.put_u128(self);
    }
}
