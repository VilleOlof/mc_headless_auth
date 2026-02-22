use std::{io::Read, net::TcpStream};

use aes::cipher::BlockDecryptMut;

use crate::minecraft::encrypt::Aes128CfbDec;

pub mod legacy_ping;
pub mod login;
pub mod status;
pub mod transfer;

fn empty_stream(stream: &mut TcpStream, dec: &mut Aes128CfbDec) {
    stream.set_nonblocking(true).unwrap();
    let mut buf = [0u8; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(read) => {
                // advance the decryptor
                let mut data = buf.to_vec();
                data.truncate(read);
                let (chunks, rest) =
                    aes::cipher::inout::InOutBuf::from(&mut data[..]).into_chunks();
                assert!(rest.is_empty());
                dec.decrypt_blocks_inout_mut(chunks);
                continue;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => panic!("{e:?}"),
        }
    }
    stream.set_nonblocking(false).unwrap();
}
