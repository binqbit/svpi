use std::str::FromStr;

use argon2::password_hash::rand_core::RngCore;
use borsh::{BorshDeserialize, BorshSerialize};
use borsh_derive::{BorshDeserialize, BorshSerialize};
use chacha20poly1305::aead::OsRng;
use sha2::{Digest, Sha256};

use crate::{
    seg_mgr::{Data, DataError, DataType, DATA_FINGERPRINT_SIZE},
    utils::crypto::{
        decrypt, derive_encryption_key, encrypt, password_hash, KdfParams, DEFAULT_PASSWORD_SALT,
        HARDENED_KDF_M_COST_KIB, HARDENED_KDF_P_COST, HARDENED_KDF_T_COST, LOW_KDF_M_COST_KIB,
        LOW_KDF_P_COST, LOW_KDF_T_COST, MEDIUM_KDF_M_COST_KIB, MEDIUM_KDF_P_COST,
        MEDIUM_KDF_T_COST, SALT_LEN, STRONG_KDF_M_COST_KIB, STRONG_KDF_P_COST, STRONG_KDF_T_COST,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum EncryptionLevel {
    Low,
    Medium,
    Strong,
    Hardened,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct EncryptionKey {
    pub key: Vec<u8>,
    pub salt: [u8; SALT_LEN],
    pub level: EncryptionLevel,
}

impl EncryptionLevel {
    pub const fn multiplier(self) -> u32 {
        match self {
            EncryptionLevel::Low => 1,
            EncryptionLevel::Medium => 2,
            EncryptionLevel::Strong => 4,
            EncryptionLevel::Hardened => 4,
        }
    }

    pub const fn strongest(self, other: EncryptionLevel) -> EncryptionLevel {
        if self.multiplier() >= other.multiplier() {
            self
        } else {
            other
        }
    }

    pub fn get_kdf_params(&self) -> KdfParams {
        match self {
            EncryptionLevel::Low => KdfParams {
                t_cost: LOW_KDF_T_COST,
                m_cost_kib: LOW_KDF_M_COST_KIB,
                p_cost: LOW_KDF_P_COST,
            },
            EncryptionLevel::Medium => KdfParams {
                t_cost: MEDIUM_KDF_T_COST,
                m_cost_kib: MEDIUM_KDF_M_COST_KIB,
                p_cost: MEDIUM_KDF_P_COST,
            },
            EncryptionLevel::Strong => KdfParams {
                t_cost: STRONG_KDF_T_COST,
                m_cost_kib: STRONG_KDF_M_COST_KIB,
                p_cost: STRONG_KDF_P_COST,
            },
            EncryptionLevel::Hardened => KdfParams {
                t_cost: HARDENED_KDF_T_COST,
                m_cost_kib: HARDENED_KDF_M_COST_KIB,
                p_cost: HARDENED_KDF_P_COST,
            },
        }
    }
}

impl Default for EncryptionLevel {
    fn default() -> Self {
        EncryptionLevel::Medium
    }
}

impl FromStr for EncryptionLevel {
    type Err = ();

    fn from_str(input: &str) -> Result<EncryptionLevel, Self::Err> {
        match input.to_lowercase().as_str() {
            "low" => Ok(EncryptionLevel::Low),
            "medium" => Ok(EncryptionLevel::Medium),
            "strong" => Ok(EncryptionLevel::Strong),
            "hardened" => Ok(EncryptionLevel::Hardened),
            _ => Err(()),
        }
    }
}

impl EncryptionKey {
    pub fn new(
        master: &str,
        name: &str,
        level: EncryptionLevel,
        dump_protection: EncryptionLevel,
    ) -> Self {
        let name_salt: [u8; SALT_LEN] = Sha256::digest(name).to_vec()[..SALT_LEN]
            .try_into()
            .expect("Slice with incorrect length");
        let key = derive_encryption_key(master.as_bytes(), &name_salt, dump_protection);

        let mut salt: [u8; SALT_LEN] = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);

        Self {
            key: key.to_vec(),
            salt,
            level,
        }
    }

    pub fn from(password: &str) -> Self {
        let salt: [u8; SALT_LEN] = Sha256::digest(DEFAULT_PASSWORD_SALT).to_vec()[..SALT_LEN]
            .try_into()
            .expect("Slice with incorrect length");

        Self {
            key: password.as_bytes().to_vec(),
            salt,
            level: EncryptionLevel::default(),
        }
    }

    pub fn encrypt(
        &mut self,
        password: &str,
        dump_protection: EncryptionLevel,
    ) -> Result<(), DataError> {
        let effective_level = self.level.strongest(dump_protection);
        let kdf_params = effective_level.get_kdf_params();
        let blob = encrypt(&self.key, password.as_bytes(), &kdf_params)
            .ok_or(DataError::EncryptionError)?;
        self.key = blob;
        Ok(())
    }

    pub fn decrypt(
        &mut self,
        password: &str,
        dump_protection: EncryptionLevel,
    ) -> Result<(), DataError> {
        let effective_level = self.level.strongest(dump_protection);
        let kdf_params = effective_level.get_kdf_params();
        let key = decrypt(&self.key, password.as_bytes(), &kdf_params)
            .ok_or(DataError::DecryptionError)?;
        self.key = key;
        Ok(())
    }

    pub fn get_password_fingerprint(
        &self,
        password: &str,
        dump_protection: EncryptionLevel,
    ) -> [u8; DATA_FINGERPRINT_SIZE] {
        let effective_level = self.level.strongest(dump_protection);
        let kdf_params = effective_level.get_kdf_params();
        let hash = password_hash(password.as_bytes(), &self.salt, &kdf_params);
        hash[..DATA_FINGERPRINT_SIZE]
            .try_into()
            .expect("Slice with incorrect length")
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.serialize(&mut buffer)
            .expect("Failed to serialize metadata");
        buffer
    }

    pub fn unpack(data: &[u8]) -> Result<Self, DataError> {
        Self::try_from_slice(data).map_err(|_| DataError::UnpackError)
    }
}

impl Data {
    pub fn encrypt(
        &self,
        key: &[u8],
        dump_protection: EncryptionLevel,
    ) -> Result<Vec<u8>, DataError> {
        let data = self.to_bytes()?;
        let kdf_params = dump_protection.get_kdf_params();
        encrypt(&data, key, &kdf_params).ok_or(DataError::EncryptionError)
    }
}

impl DataType {
    pub fn decrypt(
        &self,
        data: &[u8],
        key: &[u8],
        dump_protection: EncryptionLevel,
    ) -> Result<Data, DataError> {
        let kdf_params = dump_protection.get_kdf_params();
        let data = decrypt(data, key, &kdf_params).ok_or(DataError::DecryptionError)?;
        let data = self.from_bytes(&data)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encryption_key_pack_unpack_roundtrip() {
        let key = EncryptionKey {
            key: b"secret_key".to_vec(),
            salt: [7u8; SALT_LEN],
            level: EncryptionLevel::Strong,
        };

        let packed = key.pack();
        let unpacked = EncryptionKey::unpack(&packed).expect("unpack");

        assert_eq!(unpacked.key, key.key);
        assert_eq!(unpacked.salt, key.salt);
        assert_eq!(unpacked.level, key.level);
    }

    #[test]
    fn encryption_key_encrypt_decrypt_roundtrip() {
        let dump_protection = EncryptionLevel::Low;
        let mut key = EncryptionKey {
            key: b"plaintext_key_material".to_vec(),
            salt: [3u8; SALT_LEN],
            level: EncryptionLevel::Medium,
        };

        let original = key.key.clone();
        key.encrypt("pw", dump_protection).expect("encrypt");
        assert_ne!(key.key, original);

        key.decrypt("pw", dump_protection).expect("decrypt");
        assert_eq!(key.key, original);
    }

    #[test]
    fn encryption_key_decrypt_rejects_wrong_password() {
        let dump_protection = EncryptionLevel::Low;
        let mut key = EncryptionKey {
            key: b"plaintext_key_material".to_vec(),
            salt: [3u8; SALT_LEN],
            level: EncryptionLevel::Medium,
        };

        key.encrypt("pw", dump_protection).expect("encrypt");

        let res = key.decrypt("wrong", dump_protection);
        assert!(matches!(res, Err(DataError::DecryptionError)));
    }

    #[test]
    fn password_fingerprint_is_stable() {
        let dump_protection = EncryptionLevel::Low;
        let key = EncryptionKey {
            key: b"key".to_vec(),
            salt: [1u8; SALT_LEN],
            level: EncryptionLevel::Low,
        };

        let a = key.get_password_fingerprint("pw", dump_protection);
        let b = key.get_password_fingerprint("pw", dump_protection);
        assert_eq!(a, b);

        let c = key.get_password_fingerprint("pw2", dump_protection);
        assert_ne!(a, c);
    }
}
