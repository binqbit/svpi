use crate::{
    pass_mgr::{PasswordManager, PasswordManagerError},
    seg_mgr::{Data, DataType, MASTER_PASSWORD_HASH_SIZE},
    utils::crypto::{
        derive_encryption_key, get_master_password_check, password_hash, DEFAULT_PASSWORD_HASH_SALT,
    },
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
        let password_fingerprint = password_hash(password.as_bytes(), DEFAULT_PASSWORD_HASH_SALT);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data_mgr::DataInterfaceType, seg_mgr::DataType};

    fn setup_mgr() -> PasswordManager {
        let mut mgr =
            PasswordManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.get_data_manager()
            .init_device(1024)
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
            .add_encryption_key("master", "key1", "pwd")
            .expect("add"));
        let seg = mgr.0.find_segment_by_name("key1").unwrap();
        assert_eq!(seg.info.data_type, DataType::EncryptionKey);
        assert!(seg.info.password_fingerprint.is_some());
    }
}
