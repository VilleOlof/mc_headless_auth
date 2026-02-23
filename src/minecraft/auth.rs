use std::{net::TcpStream, sync::Arc};

use aes::cipher::generic_array;
use der::Encode;
use rand::RngExt;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey, pkcs8::SubjectPublicKeyInfo};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    ServerError,
    error::TypeError,
    minecraft::{
        array::Array,
        encrypt::{Aes128CfbDec, Aes128CfbEnc, create_cipher},
        hash::notchian_digest,
        login_start::LoginStart,
        packet::{Packet, ReadPacketData},
        packets, protocol_version,
        uuid::uuid_to_bytes,
    },
};

// https://minecraft.wiki/w/Java_Edition_protocol/Encryption

const SERVER_ID: &'static str = "mc_headless_auth";

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<GameProfileProps>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct GameProfileProps {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthResponse {
    pub enc: Aes128CfbEnc,
    #[allow(unused)]
    pub dec: Aes128CfbDec,
    pub profile: GameProfile,
}

pub fn authenticate(
    stream: &mut TcpStream,
    public_key: &Arc<RsaPublicKey>,
    private_key: &Arc<RsaPrivateKey>,
    login_start: &LoginStart,
    protocol_version: i32,
) -> Result<AuthResponse, ServerError> {
    let mut encoded_public_key = Vec::new();
    SubjectPublicKeyInfo::from_key(public_key.as_ref())?.encode(&mut encoded_public_key)?;

    let mut token = vec![0; 64];
    rand::rng().fill(&mut token);

    packets::encryption_request(
        SERVER_ID,
        encoded_public_key.clone(),
        token.clone(),
        protocol_version > protocol_version::V1_20_5,
    )
    .write_stream(stream)?;

    // 6. Encryption Response
    let mut response = Packet::from_stream(stream, 0x01)?;
    let response = EncryptionResponse::read(&mut response.data)?;

    let shared_secret = private_key.decrypt(Pkcs1v15Encrypt, response.shared_secret.as_ref())?;
    let verify_token = private_key.decrypt(Pkcs1v15Encrypt, response.verify_token.as_ref())?;

    if token != verify_token {
        // grab a sample for the error on the last 4 bytes
        // if they are different they are probably different
        return Err(ServerError::MismatchedVerifyTokens(
            token[token.len() - 4..].to_vec(),
            verify_token[verify_token.len() - 4..].to_vec(),
        ));
    }

    let server_hash = gen_server_hash(&shared_secret, &encoded_public_key);
    let profile = has_joined(&login_start.name.0, &server_hash)?;
    if profile.name != login_start.name.0 {
        return Err(ServerError::MismatchedUsernames(
            login_start.name.0.clone(),
            profile.name,
        ));
    }

    let (mut enc, mut dec) = create_cipher(&shared_secret)?;

    packets::set_compression(0).write_encrypted_stream(stream, &mut enc)?;
    packets::login_success(
        uuid_to_bytes(profile.id, protocol_version),
        &profile.name,
        vec![],
    )
    .write_compressed_encrypted_stream(stream, &mut enc)?;

    let _ = Packet::from_compressed_encrypted_stream(stream, &mut dec, 0x03)?;

    return Ok(AuthResponse { enc, dec, profile });
}

const MOJANG_HAS_JOINED_URL: &'static str =
    "https://sessionserver.mojang.com/session/minecraft/hasJoined";
fn has_joined(username: &str, hash: &str) -> Result<GameProfile, ServerError> {
    let url = format!("{MOJANG_HAS_JOINED_URL}?username={username}&serverId={hash}",);
    let response = reqwest::blocking::get(&url)?;

    let game_profile: GameProfile = response.json()?;

    Ok(game_profile)
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
    fn read(data: &mut bytes::Bytes) -> Result<Self, TypeError> {
        let shared = Array::<u8>::read(data)?;
        let token = Array::<u8>::read(data)?;

        Ok(Self {
            shared_secret: shared,
            verify_token: token,
        })
    }
}
