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
