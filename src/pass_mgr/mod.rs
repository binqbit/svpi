use thiserror::Error;

use crate::{
    data_mgr::{DataInterfaceType, DeviceError},
    seg_mgr::{DataError, DataManagerError, SegmentError, SegmentManager},
};

mod data;
mod encryption;
mod password;

#[derive(Debug, Error)]
pub enum PasswordManagerError {
    #[error("Encryption error: {0}")]
    EncryptionError(DataError),
    #[error("Data error: {0}")]
    DataError(DataError),

    #[error("Get encryption key error: {0}")]
    GetEncryptionKey(SegmentError),
    #[error("Invalid encryption key: {0}")]
    InvalidEncryptionKey(DataError),

    #[error("Set master password error: {0}")]
    SetMasterPassword(DeviceError),
    #[error("Error resetting master password: {0}")]
    ResetMasterPassword(DeviceError),
    #[error("Add encryption key error: {0}")]
    AddEncryptionKey(SegmentError),

    #[error("Save password error: {0}")]
    SavePasswordError(SegmentError),
    #[error("Read password error: {0}")]
    ReadPasswordError(SegmentError),
    #[error("Remove password error: {0}")]
    RemovePasswordError(SegmentError),
    #[error("Rename password error: {0}")]
    RenamePasswordError(SegmentError),
    #[error("Change data type error: {0}")]
    ChangeDataTypeError(SegmentError),
}

pub struct PasswordManager(pub SegmentManager);

impl PasswordManager {
    pub fn from_device_type(device_type: DataInterfaceType) -> Result<Self, DataManagerError> {
        SegmentManager::from_device_type(device_type).map(Self)
    }

    pub fn try_load(device_type: DataInterfaceType) -> Result<Self, DataManagerError> {
        SegmentManager::try_load(device_type).map(Self)
    }

    pub fn get_data_manager(&mut self) -> &mut SegmentManager {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{data_mgr::DataInterfaceType, pass_mgr::PasswordManager, seg_mgr::EncryptionLevel};

    #[test]
    fn test_init() {
        let mut pass_mgr = PasswordManager::from_device_type(DataInterfaceType::Memory(vec![]))
            .expect("Init pass mgr");

        pass_mgr
            .get_data_manager()
            .init_device(1024, EncryptionLevel::Low)
            .expect("Init device");

        assert_eq!(pass_mgr.0.metadata.memory_size, 1024);
    }

    #[test]
    fn test_dump() {
        let mut pass_mgr = PasswordManager::from_device_type(DataInterfaceType::Memory(vec![]))
            .expect("Init pass mgr");

        pass_mgr
            .get_data_manager()
            .init_device(1024, EncryptionLevel::Low)
            .expect("Init device");

        let dump = pass_mgr.get_data_manager().get_dump().expect("Dump data");

        let pass_mgr =
            PasswordManager::try_load(DataInterfaceType::Memory(dump)).expect("Init pass mgr");

        assert_eq!(pass_mgr.0.metadata.memory_size, 1024);
    }
}
