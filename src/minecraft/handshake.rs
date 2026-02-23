use bytes::{Buf, Bytes};

use crate::{
    error::TypeError,
    minecraft::{packet::ReadPacketData, string::PacketString, var_int::VarInt},
};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct Handshake {
    pub protocol_version: VarInt,
    pub server_address: PacketString,
    pub server_port: u16,
    pub intent: Intent,
}

#[derive(Debug, Clone)]
#[repr(i32)]
pub enum Intent {
    Status = 1,
    Login = 2,
    Transfer = 3,
    Unknown(i32),
}

impl Intent {
    fn from_i32(val: i32) -> Intent {
        match val {
            1 => Intent::Status,
            2 => Intent::Login,
            3 => Intent::Transfer,
            val => Intent::Unknown(val),
        }
    }
}

impl ReadPacketData for Handshake {
    fn read(data: &mut Bytes) -> Result<Self, TypeError> {
        let p_version = VarInt::read(data)?;
        let s_addr = PacketString::read(data)?;
        let s_port = data.get_u16();
        let intent = VarInt::read(data)?;

        Ok(Handshake {
            protocol_version: p_version,
            server_address: s_addr,
            server_port: s_port,
            intent: Intent::from_i32(intent.0),
        })
    }
}
