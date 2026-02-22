use std::{io::Write, net::TcpStream};

use bytes::{BufMut, BytesMut};

const DESC_WARNING: &'static str = "Use a client newer than 1.7+";

pub fn match_id_to_version(id: i32, stream: &mut TcpStream) {
    match id {
        -127 => _1dot6::advance(stream),             // FE 01 FA
        -126 => _1dot4_to_1dot5::advance(stream),    // FE 01
        -125 => beta1dot8_to_1dot3::advance(stream), // FE
        _ => panic!("unknown custom packet id for legacy pings"),
    }
}

pub fn advance(stream: &mut TcpStream) {
    let mut data = BytesMut::new();

    let str: Vec<u16> = format!("§1\0127\01.4.2\0{DESC_WARNING}\00\00",)
        .encode_utf16()
        .collect();
    // let str: Vec<u16> = format!(
    //     "§1\0127\01.4.2\0{}\00\00",
    //     config.legacy_decription.unwrap_or_default()
    // )
    // .encode_utf16()
    // .collect();

    data.put_u8(0xFF);
    data.put_u16(str.len() as u16);
    for unit in str {
        data.put_u16(unit);
    }

    println!("{:?}", data);

    stream.write_all(&data).unwrap();
}

mod _1dot6 {
    use std::net::TcpStream;

    pub fn advance(stream: &mut TcpStream) {
        todo!()
    }
}

mod _1dot4_to_1dot5 {
    use std::net::TcpStream;

    pub fn advance(stream: &mut TcpStream) {
        todo!()
    }
}

mod beta1dot8_to_1dot3 {
    use std::net::TcpStream;

    pub fn advance(stream: &mut TcpStream) {
        todo!()
    }
}
