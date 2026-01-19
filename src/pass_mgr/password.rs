use crate::{
    pass_mgr::{PasswordManager, PasswordManagerError},
    seg_mgr::{DataType, EncryptionKey, EncryptionLevel, SegmentError, MASTER_PASSWORD_HASH_SIZE},
    utils::crypto::get_master_password_check,
};

impl PasswordManager {
    pub fn set_master_password(&mut self, password: &str) -> Result<(), PasswordManagerError> {
        let hash = get_master_password_check(password.as_bytes(), self.0.metadata.dump_protection);
        self.0
            .set_master_password_hash(hash)
            .map_err(PasswordManagerError::SetMasterPassword)
    }

    pub fn reset_master_password(&mut self) -> Result<(), PasswordManagerError> {
        self.0
            .reset_master_password_hash()
            .map_err(PasswordManagerError::ResetMasterPassword)
    }

    pub fn is_master_password_set(&self) -> bool {
        !self
            .0
            .check_master_password_hash([0; MASTER_PASSWORD_HASH_SIZE])
    }

    pub fn check_master_password(&self, password: &str) -> bool {
        let hash = get_master_password_check(password.as_bytes(), self.0.metadata.dump_protection);
        self.0.check_master_password_hash(hash)
    }

    pub fn add_encryption_key(
        &mut self,
        master_password: &str,
        name: &str,
        password: &str,
        level: EncryptionLevel,
    ) -> Result<bool, PasswordManagerError> {
        let dump_protection = self.0.metadata.dump_protection;
        let mut encryption_key = EncryptionKey::new(master_password, name, level, dump_protection);
        encryption_key
            .encrypt(password, dump_protection)
            .map_err(PasswordManagerError::EncryptionError)?;
        let data = encryption_key.pack();

        let password_hash = encryption_key.get_password_fingerprint(password, dump_protection);

        self.0
            .set_segment(name, &data, DataType::EncryptionKey, Some(password_hash))
            .map(|seg| seg.is_some())
            .map_err(PasswordManagerError::AddEncryptionKey)
    }

    pub fn link_key(&mut self, name: &str, password: &str) -> Result<(), PasswordManagerError> {
        let dump_protection = self.0.metadata.dump_protection;
        let (fingerprint, key) = self.get_encryption_key(password, None)?;

        let segment = if let Some(segment) = self.0.find_segment_by_name(name) {
            segment
        } else {
            return Err(PasswordManagerError::ReadPasswordError(
                SegmentError::NotFound(name.to_string()),
            ));
        };

        let data_type = segment.info.data_type;
        let data = segment
            .read_data()
            .map_err(PasswordManagerError::ReadPasswordError)?
            .to_bytes()
            .map_err(|err| PasswordManagerError::ReadPasswordError(SegmentError::DataError(err)))?;
        let _ = data_type
            .decrypt(&data, &key, dump_protection)
            .map_err(PasswordManagerError::InvalidEncryptionKey)?;

        segment.info.password_fingerprint = Some(fingerprint);
        segment
            .update_meta()
            .map_err(PasswordManagerError::SavePasswordError)?;

        Ok(())
    }

    pub fn sync_encryption_keys(
        &mut self,
        master_password: &str,
    ) -> Result<(), PasswordManagerError> {
        let dump_protection = self.0.metadata.dump_protection;
        let mut encryption_keys = vec![];
        for key in self.get_encryption_keys() {
            let encryption_key = EncryptionKey::unpack(
                &key.read_data()
                    .map_err(PasswordManagerError::ReadPasswordError)?
                    .to_bytes()
                    .map_err(|err| {
                        PasswordManagerError::ReadPasswordError(SegmentError::DataError(err))
                    })?,
            )
            .map_err(PasswordManagerError::InvalidEncryptionKey)?;

            let encryption_key = EncryptionKey::new(
                master_password,
                key.get_name().as_str(),
                encryption_key.level,
                dump_protection,
            );

            encryption_keys.push((encryption_key, key.info.fingerprint.fingerprint));
        }

        let segments = self
            .0
            .get_active_segments_mut()
            .into_iter()
            .filter(|seg| seg.info.data_type != DataType::EncryptionKey)
            .collect::<Vec<_>>();

        for segment in segments {
            let data_type = segment.info.data_type;
            let data = segment
                .read_data()
                .map_err(PasswordManagerError::ReadPasswordError)?
                .to_bytes()
                .map_err(|err| {
                    PasswordManagerError::ReadPasswordError(SegmentError::DataError(err))
                })?;

            for (encryption_key, fingerprint) in &encryption_keys {
                if data_type
                    .decrypt(&data, &encryption_key.key, dump_protection)
                    .is_ok()
                {
                    segment.info.password_fingerprint = Some(*fingerprint);
                    segment
                        .update_meta()
                        .map_err(PasswordManagerError::SavePasswordError)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data_mgr::DataInterfaceType, seg_mgr::DataType};

    fn setup_mgr() -> PasswordManager {
        let mut mgr =
            PasswordManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.get_data_manager()
            .init_device(1024, EncryptionLevel::Low)
            .expect("init device");
        mgr
    }

    #[test]
    fn master_password_cycle() {
        let mut mgr = setup_mgr();
        assert!(!mgr.is_master_password_set());
        mgr.set_master_password("master").expect("set");
        assert!(mgr.is_master_password_set());
        assert!(mgr.check_master_password("master"));
        mgr.reset_master_password().expect("reset");
        assert!(!mgr.is_master_password_set());
    }

    #[test]
    fn add_encryption_key_creates_segment() {
        let mut mgr = setup_mgr();
        mgr.set_master_password("master").unwrap();
        assert!(mgr
            .add_encryption_key("master", "key1", "pwd", EncryptionLevel::default())
            .expect("add"));
        let seg = mgr.0.find_segment_by_name("key1").unwrap();
        assert_eq!(seg.info.data_type, DataType::EncryptionKey);
        assert!(seg.info.password_fingerprint.is_some());
    }

    #[test]
    fn link_key_hash_to_data() {
        let mut mgr = setup_mgr();
        mgr.set_master_password("master").unwrap();
        mgr.add_encryption_key("master", "key1", "pwd", EncryptionLevel::default())
            .unwrap();
        let old_key_fp = mgr
            .0
            .find_segment_by_name("key1")
            .unwrap()
            .info
            .fingerprint
            .fingerprint;

        mgr.save_password("test", "data", Some("pwd".to_string()))
            .unwrap();

        mgr.remove_password("key1").unwrap();
        mgr.add_encryption_key("master", "key1", "pwd", EncryptionLevel::default())
            .unwrap();
        let new_key_fp = mgr
            .0
            .find_segment_by_name("key1")
            .unwrap()
            .info
            .fingerprint
            .fingerprint;

        mgr.link_key("test", "pwd").unwrap();
        let new_data_fp = mgr
            .0
            .find_segment_by_name("test")
            .unwrap()
            .info
            .password_fingerprint
            .unwrap();

        assert_ne!(old_key_fp, new_key_fp);
        assert_eq!(new_data_fp, new_key_fp);
    }

    #[test]
    fn sync_keys_hash_to_data() {
        let mut mgr = setup_mgr();
        mgr.set_master_password("master").unwrap();
        mgr.add_encryption_key("master", "key1", "pwd", EncryptionLevel::default())
            .unwrap();
        let old_key_fp = mgr
            .0
            .find_segment_by_name("key1")
            .unwrap()
            .info
            .fingerprint
            .fingerprint;

        mgr.save_password("test", "data", Some("pwd".to_string()))
            .unwrap();

        mgr.remove_password("key1").unwrap();
        mgr.add_encryption_key("master", "key1", "pwd", EncryptionLevel::default())
            .unwrap();
        let new_key_fp = mgr
            .0
            .find_segment_by_name("key1")
            .unwrap()
            .info
            .fingerprint
            .fingerprint;

        mgr.sync_encryption_keys("master").unwrap();
        let new_data_fp = mgr
            .0
            .find_segment_by_name("test")
            .unwrap()
            .info
            .password_fingerprint
            .unwrap();

        assert_ne!(old_key_fp, new_key_fp);
        assert_eq!(new_data_fp, new_key_fp);
    }
}
