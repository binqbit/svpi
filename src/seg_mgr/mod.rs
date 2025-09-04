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
pub use metadata::*;
pub use segment::*;

pub const ARCHITECTURE_VERSION: u32 = 6;
pub const METADATA_SIZE: usize = std::mem::size_of::<Metadata>();
pub const SEGMENT_INFO_SIZE: usize = std::mem::size_of::<DataInfo>();

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

    pub fn init_device(&mut self, memory_size: u32) -> Result<(), DataManagerError> {
        self.metadata.memory_size = memory_size;
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

        // TODO: Implement error handling
        seg_mgr.load_metadata().unwrap();

        seg_mgr
            .load_segments()
            .map_err(DataManagerError::DeviceError)?;

        Ok(seg_mgr)
    }
}
