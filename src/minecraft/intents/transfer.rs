use std::net::TcpStream;

use crate::{
    message::MessageGenerator,
    minecraft::{handshake::Handshake, server::ConnectionState},
    token::TokenGenerator,
};

pub fn advance<T: TokenGenerator, M: MessageGenerator>(
    stream: &mut TcpStream,
    state: ConnectionState<T, M>,
    handshake: Handshake,
) {
    todo!()
}
