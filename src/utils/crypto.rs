extern crate ring;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce};

use crate::seg_mgr::EncryptionLevel;

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 24;
pub const KDF_OUTPUT_LEN: usize = 32;

pub const LOW_KDF_P_COST: u32 = 1;
pub const LOW_KDF_T_COST: u32 = 1;
#[cfg(not(test))]
pub const LOW_KDF_M_COST_KIB: u32 = 32_768;

pub const MEDIUM_KDF_P_COST: u32 = 2;
pub const MEDIUM_KDF_T_COST: u32 = 1;
#[cfg(not(test))]
pub const MEDIUM_KDF_M_COST_KIB: u32 = 131_072;

pub const STRONG_KDF_P_COST: u32 = 4;
pub const STRONG_KDF_T_COST: u32 = 2;
#[cfg(not(test))]
pub const STRONG_KDF_M_COST_KIB: u32 = 262_144;

pub const HARDENED_KDF_P_COST: u32 = 4;
pub const HARDENED_KDF_T_COST: u32 = 4;
#[cfg(not(test))]
pub const HARDENED_KDF_M_COST_KIB: u32 = 262_144;

#[cfg(test)]
pub const LOW_KDF_M_COST_KIB: u32 = 64;
#[cfg(test)]
pub const MEDIUM_KDF_M_COST_KIB: u32 = 64;
#[cfg(test)]
pub const STRONG_KDF_M_COST_KIB: u32 = 64;
#[cfg(test)]
pub const HARDENED_KDF_M_COST_KIB: u32 = 64;

pub const MASTER_PASSWORD_CHECK_SALT: &[u8] = b"\0master_password_check\0";
pub const DEFAULT_PASSWORD_SALT: &[u8] = b"\0default_password\0";

#[derive(Debug, Clone, Copy)]
pub struct KdfParams {
    pub m_cost_kib: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl KdfParams {
    pub fn with_protection_level(self, protection: EncryptionLevel) -> Self {
        let multiplier = protection.multiplier();
        let max = match protection {
            EncryptionLevel::Low => 2,
            EncryptionLevel::Medium => 16,
            EncryptionLevel::Strong => 16,
            EncryptionLevel::Hardened => 16,
        };
        Self {
            m_cost_kib: self.m_cost_kib,
            t_cost: self.t_cost.saturating_mul(multiplier).min(max),
            p_cost: self.p_cost.saturating_mul(multiplier),
        }
    }
}

pub fn encrypt(data: &[u8], password: &[u8], kdf_params: &KdfParams) -> Option<Vec<u8>> {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);

    let params = Params::new(
        kdf_params.m_cost_kib,
        kdf_params.t_cost,
        kdf_params.p_cost,
        Some(KDF_OUTPUT_LEN),
    )
    .unwrap();
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

pub fn decrypt(blob: &[u8], password: &[u8], kdf_params: &KdfParams) -> Option<Vec<u8>> {
    if blob.len() < SALT_LEN + NONCE_LEN {
        return None;
    }
    let (salt, rest) = blob.split_at(SALT_LEN);
    let (nonce, ciphertext) = rest.split_at(NONCE_LEN);

    let params = Params::new(
        kdf_params.m_cost_kib,
        kdf_params.t_cost,
        kdf_params.p_cost,
        Some(KDF_OUTPUT_LEN),
    )
    .unwrap();
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

pub fn password_hash(password: &[u8], salt: &[u8], kdf_params: &KdfParams) -> [u8; KDF_OUTPUT_LEN] {
    let params = Params::new(
        kdf_params.m_cost_kib,
        kdf_params.t_cost,
        kdf_params.p_cost,
        Some(KDF_OUTPUT_LEN),
    )
    .expect("Invalid Argon2 params");

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; KDF_OUTPUT_LEN];
    argon2
        .hash_password_into(password, &salt, &mut output)
        .expect("Argon2 hashing failed");

    output
}

pub fn get_master_password_check(
    master_password: &[u8],
    dump_protection: EncryptionLevel,
) -> [u8; KDF_OUTPUT_LEN] {
    let kdf_params = EncryptionLevel::Hardened
        .get_kdf_params()
        .with_protection_level(dump_protection);
    password_hash(master_password, MASTER_PASSWORD_CHECK_SALT, &kdf_params)
}

pub fn derive_encryption_key(
    master_password: &[u8],
    salt: &[u8],
    dump_protection: EncryptionLevel,
) -> [u8; KDF_OUTPUT_LEN] {
    let kdf_params = EncryptionLevel::Hardened
        .get_kdf_params()
        .with_protection_level(dump_protection);
    password_hash(master_password, salt, &kdf_params)
}
