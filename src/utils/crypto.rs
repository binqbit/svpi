extern crate ring;

use ring::rand::SecureRandom;
use ring::{aead, pbkdf2, rand};
use std::num::NonZeroU32;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const ITERATIONS: u32 = 1_000_000;

fn gen_salt() -> [u8; SALT_LEN] {
    let rng = rand::SystemRandom::new();
    let mut salt = [0u8; SALT_LEN];
    rng.fill(&mut salt).unwrap();
    salt
}

fn gen_key(password: &[u8], salt: &[u8]) -> Result<aead::LessSafeKey, ring::error::Unspecified> {
    let mut key = [0u8; KEY_LEN];
    let iterations = NonZeroU32::new(ITERATIONS).unwrap();
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        password,
        &mut key,
    );
    let sealing_key = aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_256_GCM, &key)?);
    Ok(sealing_key)
}

fn gen_nonce() -> [u8; NONCE_LEN] {
    let rng = rand::SystemRandom::new();
    let mut nonce = [0u8; NONCE_LEN];
    rng.fill(&mut nonce).unwrap();
    nonce
}

fn get_salt(encrypted_data: &[u8]) -> Result<[u8; SALT_LEN], ring::error::Unspecified> {
    if encrypted_data.len() < SALT_LEN {
        return Err(ring::error::Unspecified);
    }
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&encrypted_data[..SALT_LEN]);
    Ok(salt)
}

fn get_nonce(encrypted_data: &[u8]) -> Result<[u8; NONCE_LEN], ring::error::Unspecified> {
    if encrypted_data.len() < SALT_LEN + NONCE_LEN {
        return Err(ring::error::Unspecified);
    }
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&encrypted_data[SALT_LEN..SALT_LEN + NONCE_LEN]);
    Ok(nonce)
}

fn get_ciphertext(encrypted_data: &[u8]) -> Result<&[u8], ring::error::Unspecified> {
    if encrypted_data.len() < SALT_LEN + NONCE_LEN {
        return Err(ring::error::Unspecified);
    }
    Ok(&encrypted_data[SALT_LEN + NONCE_LEN..])
}

pub fn encrypt(data: &[u8], password: &[u8]) -> Result<Vec<u8>, ring::error::Unspecified> {
    let salt = gen_salt();
    let sealing_key = gen_key(password, &salt)?;
    let nonce = gen_nonce();

    let mut in_out = data.to_vec();
    sealing_key.seal_in_place_append_tag(
        aead::Nonce::assume_unique_for_key(nonce),
        aead::Aad::empty(),
        &mut in_out,
    )?;

    let mut result = salt.to_vec();
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&in_out);
    Ok(result)
}

pub fn decrypt(
    encrypted_data: &[u8],
    password: &[u8],
) -> Result<Vec<u8>, ring::error::Unspecified> {
    let salt = get_salt(encrypted_data)?;
    let nonce = get_nonce(encrypted_data)?;
    let ciphertext = get_ciphertext(encrypted_data)?;
    let opening_key = gen_key(password, &salt)?;

    let mut in_out = ciphertext.to_vec();
    opening_key.open_in_place(
        aead::Nonce::assume_unique_for_key(nonce),
        aead::Aad::empty(),
        &mut in_out,
    )?;

    in_out.truncate(in_out.len() - aead::AES_256_GCM.tag_len());
    Ok(in_out)
}
