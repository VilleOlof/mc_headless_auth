use std::{
    net::TcpListener,
    sync::Arc,
    thread::{self, spawn},
    time::Duration,
};

use crossbeam::channel::Sender;
use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::{
    channel_message::ChannelMessage,
    config::ServerConfig,
    message::MessageGenerator,
    minecraft::{
        auth::gen_rsa_key,
        handshake::{Handshake, Intent},
        intents,
        packet::{Packet, ReadPacketData},
    },
    token::TokenGenerator,
};

#[derive(Debug, Clone)]
pub struct ConnectionState<T: TokenGenerator, M: MessageGenerator> {
    pub public_key: Arc<RsaPublicKey>,
    pub private_key: Arc<RsaPrivateKey>,
    pub sender: Sender<ChannelMessage>,
    pub token: T,
    pub message: M,
}

pub fn start(config: ServerConfig, s: Sender<ChannelMessage>) -> ! {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).unwrap();

    let (priv_key, pub_key) = gen_rsa_key();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        stream.set_nodelay(true).unwrap();

        let state = ConnectionState {
            public_key: pub_key.clone(),
            private_key: priv_key.clone(),
            sender: s.clone(),
            token: config.token.clone(),
            message: config.message.clone(),
        };
        let status_config = config.status.clone();

        spawn(move || {
            println!("new connection: {}", stream.peer_addr().unwrap());

            // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#What's_the_normal_login_sequence_for_a_client?
            // 1. Handshake
            let mut packet = Packet::from_stream(&mut stream);
            if packet.id.0 < 0 {
                intents::legacy_ping::match_id_to_version(packet.id.0, &mut stream);
            } else {
                let handshake = Handshake::read(&mut packet.data);
                println!("{handshake:?}");

                match handshake.intent {
                    Intent::Status => {
                        intents::status::advance(&mut stream, state, handshake, status_config)
                    }
                    Intent::Login => intents::login::advance(&mut stream, state, handshake),
                    Intent::Transfer => intents::transfer::advance(&mut stream, state, handshake),
                    Intent::Unknown(intent) => {
                        eprintln!("unknown intent in handshake: {:?}", intent);
                    }
                }
            }

            // wait for the client to like be happy & then disconnect it
            thread::sleep(Duration::from_secs(5));
            stream.shutdown(std::net::Shutdown::Both).unwrap();
        });
    }

    panic!()
}
