use std::net::TcpStream;

use simdnbt::owned::{NbtList, NbtTag};

use crate::{
    ServerError, User,
    channel_message::{ChannelMessage, MessageData},
    message::MessageGenerator,
    minecraft::{
        auth::authenticate,
        encrypt::Aes128CfbEnc,
        handshake::Handshake,
        login_start::LoginStart,
        packet::{Packet, ReadPacketData},
        packets,
        server::ConnectionState,
    },
    token::TokenGenerator,
};

pub fn advance<T: TokenGenerator, M: MessageGenerator>(
    stream: &mut TcpStream,
    state: ConnectionState<T, M>,
    handshake: Handshake,
) -> Result<(), ServerError> {
    let mut packet = Packet::from_stream(stream, 0x00)?;
    let login_start = LoginStart::read(&mut packet.data)?;

    let mut auth_res = authenticate(
        stream,
        &state.public_key,
        &state.private_key,
        &login_start,
        handshake.protocol_version.0,
    )?;

    let user = User {
        username: auth_res.profile.name,
        uuid: auth_res.profile.id,
    };
    let token = send_disconnect(stream, state.clone(), &user, &mut auth_res.enc)?;

    state
        .broadcast
        .send(ChannelMessage::new(MessageData::OnJoin {
            user,
            token: token.to_string(),
        }));

    Ok(())
}

fn send_disconnect<T: TokenGenerator, M: MessageGenerator>(
    stream: &mut TcpStream,
    state: ConnectionState<T, M>,
    user: &User,
    enc: &mut Aes128CfbEnc,
) -> Result<String, ServerError> {
    let gen_token = state.token.generate(&user);
    let msg = state
        .message
        .create_message(&state.token.display(&gen_token));

    // text component must be either a compound, list of compounds, or a plain string
    // we must verify because the message can be customized from the consumer
    match msg {
        NbtTag::Compound(_) => (),
        NbtTag::List(NbtList::Compound(_)) => (),
        NbtTag::String(_) => (),
        _ => return Err(ServerError::InvalidMessageNbtTag(msg)),
    }

    packets::disconnect_configuration(msg).write_compressed_encrypted_stream(stream, enc)?;

    Ok(gen_token)
}
