use crate::{
    pass_mgr::{PasswordManager, PasswordManagerError},
    seg_mgr::{DataType, EncryptionKey, Segment, DATA_FINGERPRINT_SIZE},
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
        password: &str,
        fingerprint: Option<[u8; DATA_FINGERPRINT_SIZE]>,
    ) -> Result<([u8; DATA_FINGERPRINT_SIZE], Vec<u8>), PasswordManagerError> {
        let dump_protection = self.0.metadata.dump_protection;
        if let Some(fp) = fingerprint {
            let encryption_key_segment = self
                .get_encryption_keys()
                .into_iter()
                .find(|seg| seg.info.fingerprint.fingerprint == fp);

            if let Some(encryption_key_segment) = encryption_key_segment {
                let data = encryption_key_segment
                    .read_data()
                    .map_err(PasswordManagerError::GetEncryptionKey)?
                    .to_bytes()
                    .map_err(PasswordManagerError::InvalidEncryptionKey)?;
                let mut key = EncryptionKey::unpack(&data)
                    .map_err(PasswordManagerError::InvalidEncryptionKey)?;
                key.decrypt(password, dump_protection)
                    .map_err(PasswordManagerError::InvalidEncryptionKey)?;
                return Ok((encryption_key_segment.info.fingerprint.fingerprint, key.key));
            }
        }

        for seg in self.get_encryption_keys() {
            let data = seg
                .read_data()
                .map_err(PasswordManagerError::GetEncryptionKey)?
                .to_bytes()
                .map_err(PasswordManagerError::InvalidEncryptionKey)?;
            let mut key =
                EncryptionKey::unpack(&data).map_err(PasswordManagerError::InvalidEncryptionKey)?;
            if key.decrypt(password, dump_protection).is_ok() {
                return Ok((seg.info.fingerprint.fingerprint, key.key));
            }
        }

        let key = EncryptionKey::from(password);
        Ok((
            key.get_password_fingerprint(password, dump_protection),
            key.key,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data_mgr::DataInterfaceType,
        seg_mgr::{DataType, EncryptionLevel},
    };

    fn setup_mgr() -> PasswordManager {
        let mut mgr =
            PasswordManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.get_data_manager()
            .init_device(1024, EncryptionLevel::Low)
            .expect("init device");
        mgr
    }

    #[test]
    fn get_encryption_key_not_found_returns_input() {
        let mut mgr = setup_mgr();
        let (fp, key) = mgr.get_encryption_key("pwd", None).unwrap();
        assert_eq!(key, b"pwd");
        assert_eq!(
            fp,
            EncryptionKey::from("pwd").get_password_fingerprint("pwd", EncryptionLevel::Low)
        );
    }

    #[test]
    fn get_encryption_key_finds_stored_key() {
        let mut mgr = setup_mgr();
        let password = "pwd";

        let dump_protection = EncryptionLevel::Low;
        let mut key = EncryptionKey::new(
            "secretkey",
            "test",
            EncryptionLevel::default(),
            dump_protection,
        );
        let expected_plain_key = key.key.clone();
        key.encrypt(password, dump_protection).expect("encrypt");
        let data = key.pack();

        mgr.0
            .set_segment(
                "enc",
                &data,
                DataType::EncryptionKey,
                Some(key.get_password_fingerprint("pwd", dump_protection)),
            )
            .expect("set segment");

        let expected_fp = crate::seg_mgr::DataFingerprint::get_fingerprint(&data);
        let (fp, key) = mgr.get_encryption_key("pwd", None).expect("get key");
        assert_eq!(fp, expected_fp);
        assert_eq!(key, expected_plain_key);

        let keys = mgr.get_encryption_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].get_name(), "enc");
    }
}
