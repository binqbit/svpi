use crate::{
    pass_mgr::{PasswordManager, PasswordManagerError},
    seg_mgr::{Data, DataType, MASTER_PASSWORD_HASH_SIZE},
    utils::crypto::{derive_encryption_key, get_master_password_check, password_hash},
};

impl PasswordManager {
    pub fn set_master_password(&mut self, password: &str) -> Result<(), PasswordManagerError> {
        let hash = get_master_password_check(password.as_bytes());
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
        let hash = get_master_password_check(password.as_bytes());
        self.0.check_master_password_hash(hash)
    }

    pub fn add_encryption_key(
        &mut self,
        master_password: &str,
        name: &str,
        password: &str,
    ) -> Result<bool, PasswordManagerError> {
        let encryption_key = derive_encryption_key(master_password.as_bytes(), name.as_bytes());
        let password_fingerprint = password_hash(password.as_bytes(), name.as_bytes());
        let data = Data::Binary(encryption_key.to_vec())
            .encrypt(password.as_bytes())
            .map_err(PasswordManagerError::EncryptionError)?;

        self.0
            .set_segment(
                name,
                &data,
                DataType::EncryptionKey,
                Some(password_fingerprint),
            )
            .map(|seg| seg.is_some())
            .map_err(PasswordManagerError::AddEncryptionKey)
    }
}
