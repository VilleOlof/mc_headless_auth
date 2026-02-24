use std::net::TcpStream;

use crate::{
    ServerError, StatusConfig,
    message::MessageGenerator,
    minecraft::{handshake::Handshake, intents::login, server::ConnectionState},
    token::TokenGenerator,
};

pub fn advance<T: TokenGenerator, M: MessageGenerator>(
    stream: &mut TcpStream,
    state: ConnectionState<T, M>,
    handshake: Handshake,
    status_config: StatusConfig,
) -> Result<(), ServerError> {
    // just handle it as a login packet
    login::advance(stream, state, handshake, status_config)
}
