use aes::{
    Aes128,
    cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, inout::InOutBuf},
};

pub type Aes128CfbEnc = cfb8::Encryptor<Aes128>;
pub type Aes128CfbDec = cfb8::Decryptor<Aes128>;

pub fn create_cipher(key: &[u8]) -> (Aes128CfbEnc, Aes128CfbDec) {
    (
        Aes128CfbEnc::new_from_slices(key, key).unwrap(),
        Aes128CfbDec::new_from_slices(key, key).unwrap(),
    )
}

pub fn encrypt_packet(cipher: &mut Aes128CfbEnc, packet: &mut [u8]) {
    let (chunks, rest) = InOutBuf::from(packet).into_chunks();
    assert!(rest.is_empty());
    cipher.encrypt_blocks_inout_mut(chunks);
}

pub fn decrypt_packet(cipher: &mut Aes128CfbDec, packet: &mut [u8]) {
    let (chunks, rest) = InOutBuf::from(packet).into_chunks();
    assert!(rest.is_empty());
    cipher.decrypt_blocks_inout_mut(chunks);
}
