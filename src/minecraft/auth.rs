use std::{net::TcpStream, str::FromStr, sync::Arc};

use aes::{Aes128, cipher::generic_array};
use bytes::{BufMut, BytesMut};
use cfb8::{Decryptor, Encryptor};
use der::Encode;
use rand::RngExt;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey, pkcs8::SubjectPublicKeyInfo};
use serde::Deserialize;
use uuid::Uuid;

use crate::minecraft::{
    array::Array,
    encrypt::{Aes128CfbDec, Aes128CfbEnc, create_cipher},
    hash::notchian_digest,
    login_start::LoginStart,
    packet::{Packet, ReadPacketData, WritePacketData},
    string::PacketString,
};

// https://minecraft.wiki/w/Java_Edition_protocol/Encryption

const SERVER_ID: &'static str = "mc_headless_auth";

#[derive(Debug, Clone, Deserialize)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    // properties: Vec<GameProfileProps>,
}

#[derive(Debug, Clone)]
pub struct AuthResponse {
    pub enc: Aes128CfbEnc,
    pub dec: Aes128CfbDec,
    pub profile: GameProfile,
}

const _1DOT20DOT5_PROTOCOL_VERSION: i32 = 766;

pub fn authenticate(
    stream: &mut TcpStream,
    public_key: &Arc<RsaPublicKey>,
    private_key: &Arc<RsaPrivateKey>,
    login_start: &LoginStart,
    protocol_version: i32,
) -> AuthResponse {
    let mut encoded_public_key = Vec::new();
    SubjectPublicKeyInfo::from_key(public_key.as_ref())
        .unwrap()
        .encode(&mut encoded_public_key)
        .unwrap();

    let mut token = vec![0; 64];
    rand::rng().fill(&mut token);

    let mut data = BytesMut::new();
    PacketString::new(SERVER_ID).write(&mut data); // id
    Array::new(encoded_public_key.clone()).write(&mut data); // public key
    Array::new(token.clone()).write(&mut data); // verify token
    // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#Offline_mode
    if protocol_version > _1DOT20DOT5_PROTOCOL_VERSION {
        data.put_u8(1); // should auth
    }

    let packet = Packet::new(0x01, data.into());
    // 4. Encryption Request
    packet.write_stream(stream);

    // 6. Encryption Response
    let mut response = Packet::from_stream(stream);
    let response = EncryptionResponse::read(&mut response.data);

    let shared_secret = private_key
        .decrypt(Pkcs1v15Encrypt, response.shared_secret.as_ref())
        .unwrap();
    let verify_token = private_key
        .decrypt(Pkcs1v15Encrypt, response.verify_token.as_ref())
        .unwrap();

    if token != verify_token {
        panic!("mismatched tokens during auth");
    }

    let server_hash = gen_server_hash(&shared_secret, &encoded_public_key);
    let profile = has_joined(&login_start.name.0, &server_hash);
    println!("profile: {}, {}", profile.id, profile.name);

    let (mut enc, mut dec) = create_cipher(&shared_secret);
    println!("switching to encrypted packets");

    let mut data = BytesMut::new();
    profile.id.as_u128().write(&mut data);
    PacketString::new(profile.name.clone()).write(&mut data);
    Array::<u8>::new(vec![]).write(&mut data); // since we dont login the player and dont use the skin we can skip this
    let packet = Packet::new(0x02, data.into());
    // 9. Login Success
    packet.write_encrypted_stream(stream, &mut enc);
    println!("sent login success");

    // 10. Login Acknowledged
    let login_ack = Packet::from_encrypted_stream(stream, &mut dec);
    if login_ack.id.0 != 0x03 {
        panic!("mismatched packet id, expected 0x03 (login_acknowledged)");
    }
    println!("got login ack, successfully authenticated");

    return AuthResponse { enc, dec, profile };
}

const MOJANG_HAS_JOINED_URL: &'static str =
    "https://sessionserver.mojang.com/session/minecraft/hasJoined";
fn has_joined(username: &str, hash: &str) -> GameProfile {
    let url = format!("{MOJANG_HAS_JOINED_URL}?username={username}&serverId={hash}",);
    let response = reqwest::blocking::get(&url).unwrap();

    println!("{url}: {}", response.status());

    let game_profile: GameProfile = response.json().unwrap();

    return game_profile;
}

fn gen_server_hash(shared_secret: &[u8], encoded_public_key: &[u8]) -> String {
    use sha1::{Digest, Sha1};

    let mut hasher = Sha1::default();
    hasher.update(SERVER_ID.as_bytes());
    hasher.update(shared_secret);
    hasher.update(encoded_public_key);

    let mut data = [0u8; 20];
    Digest::finalize_into(
        hasher,
        generic_array::GenericArray::from_mut_slice(&mut data),
    );

    return notchian_digest(data);
}

const RSA_BIT_SIZE: usize = 1024;
pub fn gen_rsa_key() -> (Arc<RsaPrivateKey>, Arc<RsaPublicKey>) {
    let mut rng = rand::rng();
    let priv_key = RsaPrivateKey::new(&mut rng, RSA_BIT_SIZE).unwrap();
    let pub_key = RsaPublicKey::from(&priv_key);

    (Arc::new(priv_key), Arc::new(pub_key))
}

#[derive(Debug, Clone)]
struct EncryptionResponse {
    shared_secret: Array<u8>,
    verify_token: Array<u8>,
}

impl ReadPacketData for EncryptionResponse {
    fn read(data: &mut bytes::Bytes) -> Self {
        let shared = Array::<u8>::read(data);
        let token = Array::<u8>::read(data);

        Self {
            shared_secret: shared,
            verify_token: token,
        }
    }
}
