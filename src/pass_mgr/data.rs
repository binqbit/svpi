use crate::{
    pass_mgr::{PasswordManager, PasswordManagerError},
    seg_mgr::{Data, DataType, SegmentError},
};

impl PasswordManager {
    pub fn save_password(
        &mut self,
        name: &str,
        password: &str,
        encryption_key: Option<String>,
    ) -> Result<bool, PasswordManagerError> {
        let dump_protection = self.0.metadata.dump_protection;
        let data = Data::from_str_infer(password);
        let data_type = data.get_type();

        let (data, password_fingerprint) = if let Some(encryption_key) = encryption_key {
            let (fingerprint, encryption_key) = self.get_encryption_key(&encryption_key, None)?;
            let data = data
                .encrypt(&encryption_key, dump_protection)
                .map_err(PasswordManagerError::EncryptionError)?;
            (data, Some(fingerprint))
        } else {
            (
                data.to_bytes().map_err(PasswordManagerError::DataError)?,
                None,
            )
        };

        self.0
            .set_segment(name, &data, data_type, password_fingerprint)
            .map(|seg| seg.is_some())
            .map_err(PasswordManagerError::SavePasswordError)
    }

    pub fn read_password<F>(
        &mut self,
        name: &str,
        get_encryption_key: F,
    ) -> Result<String, PasswordManagerError>
    where
        F: FnOnce() -> String,
    {
        let dump_protection = self.0.metadata.dump_protection;
        let segment = if let Some(segment) = self.0.find_segment_by_name(name) {
            segment
        } else {
            return Err(PasswordManagerError::ReadPasswordError(
                SegmentError::NotFound(name.to_string()),
            ));
        };

        let data = segment
            .read_data()
            .map_err(PasswordManagerError::ReadPasswordError)?;
        let data_type = segment.info.data_type;

        let password_fingerprint = segment.info.password_fingerprint;

        if password_fingerprint.is_some() {
            let data = data.to_bytes().map_err(|err| {
                PasswordManagerError::ReadPasswordError(SegmentError::DataError(err))
            })?;
            let encryption_key = get_encryption_key();
            let (_, encryption_key) =
                self.get_encryption_key(&encryption_key, password_fingerprint)?;
            let data = data_type
                .decrypt(&data, &encryption_key, dump_protection)
                .map_err(|err| {
                    PasswordManagerError::ReadPasswordError(SegmentError::DataError(err))
                })?;
            Ok(data.to_string().map_err(|err| {
                PasswordManagerError::ReadPasswordError(SegmentError::DataError(err))
            })?)
        } else {
            Ok(data.to_string().map_err(|err| {
                PasswordManagerError::ReadPasswordError(SegmentError::DataError(err))
            })?)
        }
    }

    pub fn remove_password(&mut self, name: &str) -> Result<(), PasswordManagerError> {
        if let Some(segment) = self.0.find_segment_by_name(name) {
            segment
                .remove()
                .map_err(PasswordManagerError::RemovePasswordError)?;
        }
        Ok(())
    }

    pub fn rename_password(
        &mut self,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), PasswordManagerError> {
        if let Some(segment) = self.0.find_segment_by_name(old_name) {
            segment
                .rename(new_name)
                .map_err(PasswordManagerError::RenamePasswordError)?;
        }
        Ok(())
    }

    pub fn change_data_type(
        &mut self,
        name: &str,
        new_data_type: DataType,
    ) -> Result<(), PasswordManagerError> {
        if let Some(segment) = self.0.find_segment_by_name(name) {
            segment
                .set_type(new_data_type)
                .map_err(PasswordManagerError::ChangeDataTypeError)?;
        }
        Ok(())
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
    fn save_read_plain_password() {
        let mut mgr = setup_mgr();
        assert!(mgr.save_password("name", "secret", None).unwrap());
        let read = mgr
            .read_password("name", || String::new())
            .expect("read password");
        assert_eq!(read, "secret");
    }

    #[test]
    fn save_read_encrypted_password() {
        let mut mgr = setup_mgr();
        let key = "enc_key".to_string();
        assert!(mgr
            .save_password("name", "secret", Some(key.clone()))
            .unwrap());
        let read = mgr
            .read_password("name", || key.clone())
            .expect("read password");
        assert_eq!(read, "secret");
    }

    #[test]
    fn remove_password_deletes_segment() {
        let mut mgr = setup_mgr();
        mgr.save_password("name", "secret", None).unwrap();
        mgr.remove_password("name").unwrap();
        assert!(mgr.0.find_segment_by_name("name").is_none());
    }

    #[test]
    fn rename_password_changes_name() {
        let mut mgr = setup_mgr();
        mgr.save_password("old", "data", None).unwrap();
        mgr.rename_password("old", "new").unwrap();
        assert!(mgr.0.find_segment_by_name("old").is_none());
        let data = mgr
            .read_password("new", || String::new())
            .expect("read renamed");
        assert_eq!(data, "data");
    }

    #[test]
    fn change_data_type_updates_segment() {
        let mut mgr = setup_mgr();
        mgr.save_password("hex", "hi", None).unwrap();
        mgr.change_data_type("hex", DataType::Hex).unwrap();
        let seg = mgr.0.find_segment_by_name("hex").unwrap();
        assert_eq!(seg.info.data_type, DataType::Hex);
    }
}
