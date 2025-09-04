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
        let data = Data::from_str_infer(password);
        let data_type = data.get_type();

        let (data, password_fingerprint) = if let Some(encryption_key) = encryption_key {
            let (fingerprint, encryption_key) = self.get_encryption_key(&encryption_key)?;
            let data = data
                .encrypt(&encryption_key)
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

        let is_encrypted = segment.info.password_fingerprint.is_some();

        if is_encrypted {
            let data = data.to_bytes().map_err(|err| {
                PasswordManagerError::ReadPasswordError(SegmentError::DataError(err))
            })?;
            let encryption_key = get_encryption_key();
            let (_, encryption_key) = self.get_encryption_key(&encryption_key)?;
            let data = data_type.decrypt(&data, &encryption_key).map_err(|err| {
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
