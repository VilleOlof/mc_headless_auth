use crossbeam::channel::SendError;
use rsa::pkcs8::spki;
use simdnbt::owned::NbtTag;
use thiserror::Error;

use crate::{channel_message::ChannelMessage, minecraft::packet::Packet};

/// Top-level Error structure  
#[derive(Debug, Error)]
pub enum MCHAError {
    #[error("No UUID from LoginStart(username:{0})")]
    NoUuid(String),
    #[error("{0:?}")]
    ServerError(#[from] ServerError),
}

/// Errors that happen in the Minecraft server on each connection  
///
/// Use [`Server::on_error`](crate::Server::on_error) to retreive these errors.  
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Failed to create a packet type: {0:?}")]
    TypeError(#[from] TypeError),
    #[error("Failed to encode public key: {0:?}")]
    DerError(#[from] der::Error),
    #[error("{0:?}")]
    SpkiError(#[from] spki::Error),
    #[error("{0:?}")]
    RsaError(#[from] rsa::Error),
    #[error("{0:?}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("{0:?}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("{0:?}")]
    ImageError(#[from] image::ImageError),
    #[error("Failed to shutdown stream: {0:?}")]
    FailedToShutdownStream(std::io::Error),
    #[error("Failed to create aes encryptor/decryptor: {0:?}")]
    InvalidLength(aes::cipher::InvalidLength),
    #[error("Failed to send channel message: {0:?}")]
    SendError(#[from] SendError<ChannelMessage>),
    #[error("Verify Tokens during Authentication were mismatched, sample: {0:?} != {1:?}")]
    MismatchedVerifyTokens(Vec<u8>, Vec<u8>),
    #[error("Usernames were different during Authentication: {0} != {1}")]
    MismatchedUsernames(String, String),
    #[error("Invalid NbtTag for message, must be either Compound, List(Compound) or String: {0:?}")]
    InvalidMessageNbtTag(NbtTag),
    #[error("Intent in handshake is unknown: {0}")]
    UnknownHandshakeIntent(i32),
}

/// Errors related to writing and reading packets and their data types  
#[derive(Debug, Error)]
pub enum TypeError {
    #[error("VarInt is too big, remaining value: {0}")]
    OversizedVarInt(i32),
    #[error("Packet is too big: {0} > {}", Packet::PACKET_LIMIT)]
    PacketSizeExceedsLimit(i32),
    #[error("Expected packet id: {0}, but got {1}")]
    UnexpectedPacketId(i32, i32),
    #[error("Packet in beta legacy was too big: {0} != {0}")]
    BetaLegacyPacketIsTooBig(u16, u16),

    #[error("Chunk tail was not empty when encrypting/decrypting packet, len:{0}")]
    ChunkTailIsNotEmpty(usize),

    #[error("Failed to read stream: {0:?}")]
    ReadError(std::io::Error),
    #[error("Failed to write to stream: {0:?}")]
    WriteError(std::io::Error),

    #[error("Failed to decompress packet: {0:?}")]
    DecompressError(miniz_oxide::inflate::DecompressError),

    #[error("{0:?}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
