use std::{net::TcpListener, sync::Arc, thread::spawn};

use rsa::{RsaPrivateKey, RsaPublicKey};

use crate::{
    ServerError,
    broadcast::Broadcast,
    channel_message::{ChannelMessage, MessageData},
    config::ServerConfig,
    message::MessageGenerator,
    minecraft::{
        auth::gen_rsa_key,
        handshake::{Handshake, Intent},
        intents::{self, legacy_ping},
        packet::{InitPacket, Packet, ReadPacketData},
    },
    token::TokenGenerator,
};

#[derive(Debug, Clone)]
pub struct ConnectionState<T: TokenGenerator, M: MessageGenerator> {
    pub public_key: Arc<RsaPublicKey>,
    pub private_key: Arc<RsaPrivateKey>,
    pub broadcast: Broadcast,
    pub token: T,
    pub message: M,
}

pub fn start(config: ServerConfig, broadcast: Broadcast) -> ! {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).unwrap();

    let (priv_key, pub_key) = gen_rsa_key();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        stream.set_nodelay(true).unwrap();

        let state = ConnectionState {
            public_key: pub_key.clone(),
            private_key: priv_key.clone(),
            broadcast: broadcast.clone(),
            token: config.token.clone(),
            message: config.message.clone(),
        };
        let status_config = config.status.clone();

        spawn(move || {
            let _bc = state.broadcast.clone();

            let result: Result<(), ServerError> = (|| {
                // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#What's_the_normal_login_sequence_for_a_client?
                // 1. Handshake
                let packet = Packet::init_reading(&mut stream)?;
                match packet {
                    InitPacket::V1_6 => legacy_ping::_1dot6::advance(&mut stream, status_config)?,
                    InitPacket::V1_4To1_5 => {
                        legacy_ping::_1dot4_to_1dot5::advance(&mut stream, status_config)?
                    }
                    InitPacket::Vbeta1_8To1_3 => {
                        legacy_ping::beta1dot8_to_1dot3::advance(&mut stream, status_config)?
                    }
                    InitPacket::V1_7Above(mut packet) => {
                        let handshake = Handshake::read(&mut packet.data)?;

                        match handshake.intent {
                            Intent::Status => intents::status::advance(
                                &mut stream,
                                state,
                                handshake,
                                status_config,
                            )?,
                            Intent::Login => {
                                intents::login::advance(&mut stream, state, handshake)?
                            }
                            Intent::Transfer => {
                                intents::transfer::advance(&mut stream, state, handshake)?
                            }
                            Intent::Unknown(intent) => {
                                return Err(ServerError::UnknownHandshakeIntent(intent));
                            }
                        }
                    }
                }

                // if we shutdown the stream instantly then the client gets "disconnected"
                std::thread::sleep(std::time::Duration::from_secs_f32(2.5));
                stream
                    .shutdown(std::net::Shutdown::Both)
                    .map_err(|e| ServerError::FailedToShutdownStream(e))?;

                Ok(())
            })();

            match result {
                Ok(_) => (),
                Err(e) => _bc.send(ChannelMessage::new(MessageData::ConnectionError(Box::new(
                    e,
                )))),
            }
        });
    }

    panic!()
}
