use std::net::TcpStream;

use bytes::{BufMut, Bytes, BytesMut};

use crate::{
    User,
    channel_message::{ChannelMessage, MessageData},
    message::MessageGenerator,
    minecraft::{
        auth::authenticate,
        encrypt::Aes128CfbEnc,
        handshake::Handshake,
        intents::empty_stream,
        login_start::LoginStart,
        packet::{Packet, ReadPacketData},
        server::ConnectionState,
    },
    token::TokenGenerator,
};

pub fn advance<T: TokenGenerator, M: MessageGenerator>(
    stream: &mut TcpStream,
    state: ConnectionState<T, M>,
    handshake: Handshake,
) {
    // 2. Login Start
    let mut packet = Packet::from_stream(stream);
    let login_start = LoginStart::read(&mut packet.data);
    println!("{login_start:?}");

    let auth_res = authenticate(
        stream,
        &state.public_key,
        &state.private_key,
        &login_start,
        handshake.protocol_version.0,
    );
    let (mut enc, mut dec) = (auth_res.enc, auth_res.dec);

    // empty stream so the next packet after we send a config finish
    // is the config finish ack
    empty_stream(stream, &mut dec);
    println!("emptied stream");

    // 9. Finish Configuration
    let packet = Packet::new(0x03, Bytes::new());
    packet.write_encrypted_stream(stream, &mut enc);
    println!("sent config finish");

    // 10. Acknowledge Finish Configuration
    let _ = Packet::from_encrypted_stream(stream, &mut dec);
    println!("got config finish ack");

    let user = User {
        username: auth_res.profile.name,
        uuid: auth_res.profile.id,
    };
    let token = send_disconnect(stream, state.clone(), &user, &mut enc);
    println!("sent disconnect msg with token: {token}");

    state
        .sender
        .send(ChannelMessage::new(MessageData::OnJoin {
            user,
            token: token.to_string(),
        }))
        .unwrap();
}

fn send_disconnect<T: TokenGenerator, M: MessageGenerator>(
    stream: &mut TcpStream,
    state: ConnectionState<T, M>,
    user: &User,
    enc: &mut Aes128CfbEnc,
) -> String {
    // https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disconnect_(play)

    let mut disconnect_data = BytesMut::new();
    let gen_token = state.token.generate(&user);
    let msg = state
        .message
        .create_message(&state.token.display(&gen_token));
    let mut msg_bytes = Vec::new();
    msg.write(&mut msg_bytes);
    disconnect_data.put(&msg_bytes[..]);
    let packet = Packet::new(0x20, disconnect_data.into());
    packet.write_encrypted_stream(stream, enc);

    return gen_token;
}
