use crate::{
    pass_mgr::{PasswordManager, PasswordManagerError},
    seg_mgr::{DataType, Segment, DATA_FINGERPRINT_SIZE},
    utils::crypto::{password_hash, DEFAULT_PASSWORD_HASH_SALT},
};

impl PasswordManager {
    pub fn get_encryption_keys(&mut self) -> Vec<&mut Segment> {
        self.0
            .get_active_segments_mut()
            .into_iter()
            .filter(|seg| seg.info.data_type == DataType::EncryptionKey)
            .collect()
    }

    pub fn get_encryption_key(
        &mut self,
        key: &str,
    ) -> Result<([u8; DATA_FINGERPRINT_SIZE], Vec<u8>), PasswordManagerError> {
        let fingerprint = password_hash(key.as_bytes(), DEFAULT_PASSWORD_HASH_SALT);

        let encryption_key_segment = self
            .get_encryption_keys()
            .into_iter()
            .find(|seg| seg.info.password_fingerprint == Some(fingerprint));

        if let Some(encryption_key_segment) = encryption_key_segment {
            let data = encryption_key_segment
                .read_data()
                .map_err(PasswordManagerError::GetEncryptionKey)?
                .to_bytes()
                .map_err(PasswordManagerError::InvalidEncryptionKey)?;
            let data = encryption_key_segment
                .info
                .data_type
                .decrypt(&data, key.as_bytes())
                .map_err(PasswordManagerError::InvalidEncryptionKey)?
                .to_bytes()
                .map_err(PasswordManagerError::InvalidEncryptionKey)?;
            Ok((
                encryption_key_segment.info.fingerprint.fingerprint,
                data.to_vec(),
            ))
        } else {
            Ok((fingerprint, key.as_bytes().to_vec()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data_mgr::DataInterfaceType,
        seg_mgr::{Data, DataType},
        utils::crypto::{password_hash, DEFAULT_PASSWORD_HASH_SALT},
    };

    fn setup_mgr() -> PasswordManager {
        let mut mgr =
            PasswordManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.get_data_manager()
            .init_device(1024)
            .expect("init device");
        mgr
    }

    #[test]
    fn get_encryption_key_not_found_returns_input() {
        let mut mgr = setup_mgr();
        let (fp, key) = mgr.get_encryption_key("pwd").unwrap();
        assert_eq!(key, b"pwd");
        assert_eq!(fp, password_hash(b"pwd", DEFAULT_PASSWORD_HASH_SALT));
    }

    #[test]
    fn get_encryption_key_finds_stored_key() {
        let mut mgr = setup_mgr();
        let enc_key = b"secretkey";
        let password = "pwd";
        let password_fp = password_hash(password.as_bytes(), DEFAULT_PASSWORD_HASH_SALT);
        let data = Data::Binary(enc_key.to_vec())
            .encrypt(password.as_bytes())
            .unwrap();
        mgr.0
            .set_segment("enc", &data, DataType::EncryptionKey, Some(password_fp))
            .unwrap();

        let expected_fp = crate::seg_mgr::DataFingerprint::get_fingerprint(&data);
        let (fp, key) = mgr.get_encryption_key(password).unwrap();
        assert_eq!(fp, expected_fp);
        assert_eq!(key, enc_key);

        let keys = mgr.get_encryption_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].get_name(), "enc");
    }
}
