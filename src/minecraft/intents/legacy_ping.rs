use crate::minecraft::packet::InitPacket;

pub fn compare_init_bytes(bytes: [u8; 3]) -> Option<InitPacket> {
    match bytes {
        [254, 0, 0] => Some(InitPacket::Vbeta1_8To1_3),
        [254, 1, 0] => Some(InitPacket::V1_4To1_5),
        [254, 1, 250] => Some(InitPacket::V1_6),
        _ => None,
    }
}

pub mod _1dot6 {
    use std::{io::Write, net::TcpStream};

    use bytes::{BufMut, BytesMut};
    use constcat::concat;

    use crate::{
        config::{MIN_SUPPORTED_VERSION, StatusConfig},
        error::TypeError,
    };

    /// https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping#1.6
    pub fn advance(stream: &mut TcpStream, config: StatusConfig) -> Result<(), TypeError> {
        let mut data = BytesMut::new();

        data.put_u8(0xFF);

        let str = format!(
            "§1\0{}\0{}\0{}\0{}\0{}",
            "127",
            concat!(MIN_SUPPORTED_VERSION, "+"),
            config.legacy_decription.unwrap_or_default(),
            "0",
            "0"
        );

        data.extend_from_slice(&(str.len() as u16 - 1).to_be_bytes()[..]);

        // https://github.com/Duckulus/mc-honeypot/blob/1514807e8af7f7cbfd13111fec334b9f4883b605/src/server/legacy.rs
        let utf16_be: Vec<u16> = str
            .encode_utf16()
            .collect::<Vec<u16>>()
            .iter()
            .map(|n| u16::from_be_bytes([(n & 0xFF) as u8, (n >> 8) as u8]))
            .collect();

        unsafe {
            data.extend_from_slice(utf16_be.align_to::<u8>().1);
        }

        stream
            .write_all(&data)
            .map_err(|e| TypeError::WriteError(e))?;

        Ok(())
    }
}

pub mod _1dot4_to_1dot5 {
    use std::net::TcpStream;

    use crate::{config::StatusConfig, error::TypeError, minecraft::intents::legacy_ping::_1dot6};

    pub fn advance(stream: &mut TcpStream, config: StatusConfig) -> Result<(), TypeError> {
        _1dot6::advance(stream, config)
    }
}

pub mod beta1dot8_to_1dot3 {
    use std::{io::Write, net::TcpStream};

    use bytes::{BufMut, BytesMut};

    use crate::{config::StatusConfig, error::TypeError};

    const MAX_PACKET_SIZE: u16 = 256;

    /// https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping#Beta_1.8_to_1.3
    pub fn advance(stream: &mut TcpStream, config: StatusConfig) -> Result<(), TypeError> {
        let mut data = BytesMut::new();
        data.put_u8(0x0FF); // packet id

        let str = format!("{}§0§0", config.legacy_decription.unwrap_or_default());
        let str = str.encode_utf16();

        let packet_len = str.clone().count() as u16;
        if packet_len > MAX_PACKET_SIZE {
            return Err(TypeError::BetaLegacyPacketIsTooBig(
                MAX_PACKET_SIZE,
                packet_len,
            ));
        }

        data.put_u16(packet_len);

        for unit in str.collect::<Vec<u16>>() {
            data.extend_from_slice(&unit.to_be_bytes()[..]);
        }

        stream
            .write_all(&data)
            .map_err(|e| TypeError::WriteError(e))?;

        Ok(())
    }
}
