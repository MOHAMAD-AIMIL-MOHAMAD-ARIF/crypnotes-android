use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use secrecy::SecretBox;
use thiserror::Error;

pub use crypnotes_versioning::ENCRYPTION_CONTAINER_VERSION;

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;
pub const MIN_SALT_LEN: usize = 16;

pub type SecretBytes = SecretBox<[u8]>;

#[derive(Debug, Clone, Copy)]
pub struct CryptoParams {
    pub argon_mem_mib: u32,
    pub argon_iters: u32,
    pub argon_lanes: u32,
}

impl Default for CryptoParams {
    fn default() -> Self {
        Self {
            argon_mem_mib: 64,
            argon_iters: 3,
            argon_lanes: 1,
        }
    }
}

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("salt length must be at least {MIN_SALT_LEN} bytes")]
    InvalidSaltLen,
    #[error("key length must be {KEY_LEN} bytes")]
    InvalidKeyLen,
    #[error("ciphertext is too short")]
    InvalidCiphertext,
    #[error("argon2 parameter error: {0}")]
    Argon2Params(String),
    #[error("argon2 derivation failed: {0}")]
    Argon2Derive(String),
    #[error("encryption/decryption failed")]
    Aead,
}

pub fn into_secret(bytes: Vec<u8>) -> SecretBytes {
    SecretBox::new(bytes.into_boxed_slice())
}

pub fn derive_kek(
    password: &str,
    salt: &[u8],
    params: CryptoParams,
) -> Result<Vec<u8>, CryptoError> {
    if salt.len() < MIN_SALT_LEN {
        return Err(CryptoError::InvalidSaltLen);
    }

    let mem_kib = params
        .argon_mem_mib
        .checked_mul(1024)
        .ok_or_else(|| CryptoError::Argon2Params("memory cost overflow".to_owned()))?;

    let argon_params = Params::new(
        mem_kib,
        params.argon_iters,
        params.argon_lanes,
        Some(KEY_LEN),
    )
    .map_err(|err| CryptoError::Argon2Params(err.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut output = vec![0_u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .map_err(|err| CryptoError::Argon2Derive(err.to_string()))?;

    Ok(output)
}

pub fn wrap_dek_with_kek(dek: &[u8], kek: &[u8]) -> Result<Vec<u8>, CryptoError> {
    encrypt_payload(kek, dek, b"crypnotes:vault:dek")
}

pub fn unwrap_dek_with_kek(wrapped_dek: &[u8], kek: &[u8]) -> Result<Vec<u8>, CryptoError> {
    decrypt_payload(kek, wrapped_dek, b"crypnotes:vault:dek")
}

pub fn encrypt_payload(kek: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = new_cipher(kek)?;
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CryptoError::Aead)?;

    let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt_payload(
    kek: &[u8],
    ciphertext_with_nonce: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    if ciphertext_with_nonce.len() <= NONCE_LEN {
        return Err(CryptoError::InvalidCiphertext);
    }

    let cipher = new_cipher(kek)?;
    let (nonce, ciphertext) = ciphertext_with_nonce.split_at(NONCE_LEN);
    cipher
        .decrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| CryptoError::Aead)
}

fn new_cipher(kek: &[u8]) -> Result<XChaCha20Poly1305, CryptoError> {
    if kek.len() != KEY_LEN {
        return Err(CryptoError::InvalidKeyLen);
    }
    XChaCha20Poly1305::new_from_slice(kek).map_err(|_| CryptoError::InvalidKeyLen)
}

pub fn current_encryption_container_version() -> u32 {
    ENCRYPTION_CONTAINER_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argon2_uses_fixed_output_size() {
        let salt = b"0123456789abcdef";
        let kek = derive_kek("password", salt, CryptoParams::default()).unwrap();
        assert_eq!(kek.len(), KEY_LEN);
    }

    #[test]
    fn wrap_unwrap_roundtrip() {
        let kek = vec![7_u8; KEY_LEN];
        let dek = vec![9_u8; KEY_LEN];

        let wrapped = wrap_dek_with_kek(&dek, &kek).unwrap();
        let unwrapped = unwrap_dek_with_kek(&wrapped, &kek).unwrap();

        assert_eq!(unwrapped, dek);
    }

    #[test]
    fn decrypt_rejects_modified_ciphertext() {
        let kek = vec![1_u8; KEY_LEN];
        let ciphertext = encrypt_payload(&kek, b"hello", b"aad").unwrap();

        let mut tampered = ciphertext.clone();
        let idx = tampered.len() - 1;
        tampered[idx] ^= 0xAA;

        let result = decrypt_payload(&kek, &tampered, b"aad");
        assert!(matches!(result, Err(CryptoError::Aead)));
    }

    #[test]
    fn encryption_container_version_matches_shared_constant() {
        assert_eq!(
            current_encryption_container_version(),
            ENCRYPTION_CONTAINER_VERSION
        );
    }
}
