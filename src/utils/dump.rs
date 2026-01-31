use borsh::{BorshDeserialize, BorshSerialize};
use borsh_derive::{BorshDeserialize, BorshSerialize};

use crate::{
    seg_mgr::{DataError, EncryptionLevel},
    utils::crypto::{decrypt, encrypt},
};

const DUMP_MAGIC: &[u8] = b"SDP";

pub const DUMP_PROTECTION_NONE: u8 = 0;
pub const DUMP_PROTECTION_LOW: u8 = 1;
pub const DUMP_PROTECTION_MEDIUM: u8 = 2;
pub const DUMP_PROTECTION_HIGH: u8 = 3;
pub const DUMP_PROTECTION_HARDENED: u8 = 4;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct DumpEnvelope {
    protection: u8,
    payload: Vec<u8>,
}

pub fn is_encrypted_dump(data: &[u8]) -> bool {
    data.starts_with(DUMP_MAGIC)
}

pub fn protection_code(level: EncryptionLevel) -> u8 {
    match level {
        EncryptionLevel::Low => DUMP_PROTECTION_LOW,
        EncryptionLevel::Medium => DUMP_PROTECTION_MEDIUM,
        EncryptionLevel::Strong => DUMP_PROTECTION_HIGH,
        EncryptionLevel::Hardened => DUMP_PROTECTION_HARDENED,
    }
}

fn code_to_level(code: u8) -> Result<EncryptionLevel, DataError> {
    match code {
        DUMP_PROTECTION_LOW => Ok(EncryptionLevel::Low),
        DUMP_PROTECTION_MEDIUM => Ok(EncryptionLevel::Medium),
        DUMP_PROTECTION_HIGH => Ok(EncryptionLevel::Strong),
        DUMP_PROTECTION_HARDENED => Ok(EncryptionLevel::Hardened),
        _ => Err(DataError::InvalidData),
    }
}

pub fn dump_protection(data: &[u8]) -> Result<Option<u8>, DataError> {
    if !is_encrypted_dump(data) {
        return Ok(None);
    }
    let envelope = DumpEnvelope::try_from_slice(&data[DUMP_MAGIC.len()..])
        .map_err(|_| DataError::InvalidData)?;
    let level = code_to_level(envelope.protection)?;
    Ok(Some(protection_code(level)))
}

pub fn encrypt_dump(
    data: &[u8],
    password: &str,
    protection: EncryptionLevel,
) -> Result<Vec<u8>, DataError> {
    let kdf_params = protection.get_kdf_params();
    let payload =
        encrypt(data, password.as_bytes(), &kdf_params).ok_or(DataError::EncryptionError)?;
    let envelope = DumpEnvelope {
        protection: protection_code(protection),
        payload,
    };
    let mut out = Vec::with_capacity(DUMP_MAGIC.len() + envelope.payload.len());
    out.extend_from_slice(DUMP_MAGIC);
    envelope
        .serialize(&mut out)
        .map_err(|_| DataError::InvalidData)?;
    Ok(out)
}

pub fn decrypt_dump(data: &[u8], password: &str) -> Result<(Vec<u8>, u8), DataError> {
    if !is_encrypted_dump(data) {
        return Err(DataError::InvalidData);
    }
    let envelope = DumpEnvelope::try_from_slice(&data[DUMP_MAGIC.len()..])
        .map_err(|_| DataError::InvalidData)?;
    let level = code_to_level(envelope.protection)?;
    let kdf_params = level.get_kdf_params();
    let plaintext = decrypt(&envelope.payload, password.as_bytes(), &kdf_params)
        .ok_or(DataError::DecryptionError)?;
    Ok((plaintext, protection_code(level)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_dump_roundtrip() {
        let data = b"dump payload";
        let password = "secret";

        let encrypted = encrypt_dump(data, password, EncryptionLevel::Low).expect("encrypt");
        assert!(is_encrypted_dump(&encrypted));

        let (plain, level) = decrypt_dump(&encrypted, password).expect("decrypt");
        assert_eq!(plain, data);
        assert_eq!(level, DUMP_PROTECTION_LOW);
    }

    #[test]
    fn decrypt_dump_rejects_wrong_password() {
        let data = b"dump payload";
        let encrypted = encrypt_dump(data, "pw", EncryptionLevel::Low).expect("encrypt");

        let err = decrypt_dump(&encrypted, "wrong").unwrap_err();
        assert!(matches!(err, DataError::DecryptionError));
    }

    #[test]
    fn dump_protection_reports_none_for_raw() {
        assert_eq!(dump_protection(b"raw dump").ok().flatten(), None);
    }
}
