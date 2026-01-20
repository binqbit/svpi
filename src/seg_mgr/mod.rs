use crate::{
    data_mgr::{DataInterfaceType, DataManager, DeviceError},
    seg_mgr::metadata::Metadata,
};
use thiserror::Error;

mod addresses;
mod data;
mod encryption;
mod mem_mgmt;
mod metadata;
mod segment;

pub use data::*;
pub use encryption::*;
pub use metadata::*;
pub use segment::*;

pub const ARCHITECTURE_VERSION: u32 = 8;
pub const METADATA_SIZE: usize = Metadata::SIZE;
pub const SEGMENT_INFO_SIZE: usize = DataInfo::SIZE;

#[derive(Error, Debug)]
pub enum DataManagerError {
    #[error("Device not found: {0}")]
    DeviceNotFound(DeviceError),
    #[error("Mismatch architecture version")]
    MismatchArchitectureVersion,
    #[error("Device not initialized")]
    DeviceNotInitialized,
    #[error("Device error: {0}")]
    DeviceError(DeviceError),
}

pub struct SegmentManager {
    pub data_mgr: DataManager,
    pub metadata: Metadata,
    pub segments: Vec<Segment>,
}

impl SegmentManager {
    pub fn from_data_manager(data_mgr: DataManager) -> Self {
        Self {
            data_mgr,
            metadata: Metadata::default(),
            segments: Vec::new(),
        }
    }

    pub fn from_device_type(device_type: DataInterfaceType) -> Result<Self, DataManagerError> {
        let data_mgr = device_type
            .load_data_manager()
            .map_err(DataManagerError::DeviceNotFound)?;
        Ok(Self::from_data_manager(data_mgr))
    }

    pub fn init_device(
        &mut self,
        memory_size: u32,
        dump_protection: EncryptionLevel,
    ) -> Result<(), DataManagerError> {
        self.metadata.memory_size = memory_size;
        self.metadata.dump_protection = dump_protection;
        self.init_metadata()
            .map_err(|_| DataManagerError::DeviceNotInitialized)?;
        Ok(())
    }

    pub fn try_load(device_type: DataInterfaceType) -> Result<Self, DataManagerError> {
        let mut seg_mgr = Self::from_device_type(device_type)?;

        if !seg_mgr
            .check_init_data()
            .map_err(DataManagerError::DeviceError)?
        {
            return Err(DataManagerError::DeviceNotInitialized);
        }

        if !seg_mgr
            .check_architecture_version()
            .map_err(DataManagerError::DeviceError)?
        {
            return Err(DataManagerError::MismatchArchitectureVersion);
        }

        seg_mgr
            .load_metadata()
            .map_err(|_| DataManagerError::DeviceNotInitialized)?;

        seg_mgr
            .load_segments()
            .map_err(DataManagerError::DeviceError)?;

        Ok(seg_mgr)
    }
}

mod tests {
    #[allow(unused)]
    use super::*;

    #[test]
    fn test_metadata_size() {
        let metadata = Metadata::default();
        let packed = metadata.pack();
        assert_eq!(packed.len(), METADATA_SIZE, "Metadata size mismatch");
    }

    #[test]
    fn test_data_info_size() {
        let plain = DataInfo::default();
        let plain_packed = plain.pack();
        assert_eq!(
            plain_packed.len(),
            SEGMENT_INFO_SIZE,
            "DataInfo size mismatch"
        );

        let mut encrypted = DataInfo::default();
        encrypted.password_fingerprint = Some([1, 2, 3, 4]);
        let encrypted_packed = encrypted.pack();
        assert_eq!(
            encrypted_packed.len(),
            SEGMENT_INFO_SIZE,
            "DataInfo size mismatch (encrypted)"
        );
    }
}
