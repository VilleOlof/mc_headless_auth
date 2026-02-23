use std::net::TcpStream;

use crate::{
    ServerError,
    message::MessageGenerator,
    minecraft::{handshake::Handshake, intents::login, server::ConnectionState},
    token::TokenGenerator,
};

pub fn advance<T: TokenGenerator, M: MessageGenerator>(
    stream: &mut TcpStream,
    state: ConnectionState<T, M>,
    handshake: Handshake,
) -> Result<(), ServerError> {
    // just handle it as a login packet
    login::advance(stream, state, handshake)
}
