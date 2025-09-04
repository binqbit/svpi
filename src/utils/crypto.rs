extern crate ring;

use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256};

use crate::seg_mgr::DATA_FINGERPRINT_SIZE;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;

pub const KDF_M_COST_KIB: u32 = 262_144;
pub const KDF_T_COST: u32 = 2;
pub const KDF_P_COST: u32 = 1;
pub const KDF_OUTPUT_LEN: usize = 32;

pub const KDF_M_COST_KIB_FOR_MASTER_PASSWORD: u32 = 524_288;
pub const KDF_T_COST_FOR_MASTER_PASSWORD: u32 = 2;
pub const MASTER_PASSWORD_CHECK_SALT: &[u8] = b"master_password_check";

pub const KDF_M_COST_KIB_FOR_HASH: u32 = 65_536;
pub const KDF_T_COST_FOR_HASH: u32 = 2;
pub const DEFAULT_PASSWORD_HASH_SALT: &[u8] = b"password_hash";

pub fn encrypt(data: &[u8], password: &[u8]) -> Option<Vec<u8>> {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);

    let params = Params::new(KDF_M_COST_KIB, KDF_T_COST, KDF_P_COST, Some(KDF_OUTPUT_LEN)).unwrap();
    let a2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key_bytes = [0u8; KDF_OUTPUT_LEN];
    a2.hash_password_into(password, &salt, &mut key_bytes)
        .unwrap();

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: data,
                aad: &[],
            },
        )
        .ok()?;

    key_bytes.fill(0);

    let mut blob = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(&salt);
    blob.extend_from_slice(&nonce);
    blob.extend_from_slice(&ciphertext);

    Some(blob)
}

pub fn decrypt(blob: &[u8], password: &[u8]) -> Option<Vec<u8>> {
    if blob.len() < SALT_LEN + NONCE_LEN {
        return None;
    }
    let (salt, rest) = blob.split_at(SALT_LEN);
    let (nonce, ciphertext) = rest.split_at(NONCE_LEN);

    let params = Params::new(KDF_M_COST_KIB, KDF_T_COST, KDF_P_COST, Some(KDF_OUTPUT_LEN)).unwrap();
    let a2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key_bytes = [0u8; KDF_OUTPUT_LEN];
    a2.hash_password_into(password, salt, &mut key_bytes)
        .unwrap();

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let plaintext = cipher.decrypt(
        XNonce::from_slice(nonce),
        Payload {
            msg: ciphertext,
            aad: &[],
        },
    );

    key_bytes.fill(0);

    if let Ok(plaintext) = plaintext {
        Some(plaintext)
    } else {
        None
    }
}

pub fn password_hash(password: &[u8], salt: &[u8]) -> [u8; DATA_FINGERPRINT_SIZE] {
    let salt = Sha256::digest(salt).to_vec();

    let params = Params::new(
        KDF_M_COST_KIB_FOR_HASH,
        KDF_T_COST_FOR_HASH,
        KDF_P_COST,
        Some(DATA_FINGERPRINT_SIZE),
    )
    .expect("Invalid Argon2 params");

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; DATA_FINGERPRINT_SIZE];
    argon2
        .hash_password_into(password, &salt, &mut output)
        .expect("Argon2 hashing failed");

    output
}

pub fn get_master_password_check(master_password: &[u8]) -> [u8; KDF_OUTPUT_LEN] {
    let params = Params::new(
        KDF_M_COST_KIB_FOR_MASTER_PASSWORD,
        KDF_T_COST_FOR_MASTER_PASSWORD,
        KDF_P_COST,
        Some(KDF_OUTPUT_LEN),
    )
    .expect("Invalid Argon2 params");

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; KDF_OUTPUT_LEN];
    argon2
        .hash_password_into(master_password, MASTER_PASSWORD_CHECK_SALT, &mut output)
        .expect("Argon2 hashing failed");

    output
}

pub fn derive_encryption_key(master_password: &[u8], salt: &[u8]) -> [u8; KDF_OUTPUT_LEN] {
    let salt = Sha256::digest(salt).to_vec();

    let params = Params::new(KDF_M_COST_KIB, KDF_T_COST, KDF_P_COST, Some(KDF_OUTPUT_LEN))
        .expect("Invalid Argon2 params");

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; KDF_OUTPUT_LEN];
    argon2
        .hash_password_into(master_password, &salt, &mut output)
        .expect("Argon2 hashing failed");

    output
}
