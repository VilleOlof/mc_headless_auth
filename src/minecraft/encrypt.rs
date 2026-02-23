use aes::{
    Aes128,
    cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, inout::InOutBuf},
};

use crate::{ServerError, TypeError};

pub type Aes128CfbEnc = cfb8::Encryptor<Aes128>;
pub type Aes128CfbDec = cfb8::Decryptor<Aes128>;

pub fn create_cipher(key: &[u8]) -> Result<(Aes128CfbEnc, Aes128CfbDec), ServerError> {
    Ok((
        Aes128CfbEnc::new_from_slices(key, key).map_err(|e| ServerError::InvalidLength(e))?,
        Aes128CfbDec::new_from_slices(key, key).map_err(|e| ServerError::InvalidLength(e))?,
    ))
}

#[must_use]
pub fn encrypt_packet(cipher: &mut Aes128CfbEnc, packet: &mut [u8]) -> Result<(), TypeError> {
    let (chunks, rest) = InOutBuf::from(packet).into_chunks();
    if !rest.is_empty() {
        return Err(TypeError::ChunkTailIsNotEmpty(rest.len()));
    }
    cipher.encrypt_blocks_inout_mut(chunks);
    Ok(())
}

#[must_use]
pub fn decrypt_packet(cipher: &mut Aes128CfbDec, packet: &mut [u8]) -> Result<(), TypeError> {
    let (chunks, rest) = InOutBuf::from(packet).into_chunks();
    if !rest.is_empty() {
        return Err(TypeError::ChunkTailIsNotEmpty(rest.len()));
    }
    cipher.decrypt_blocks_inout_mut(chunks);
    Ok(())
}
