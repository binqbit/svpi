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
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
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
    /// Total number of segment metadata entries stored on the device/file.
    /// Includes deleted/inactive entries until optimization compacts them.
    pub segments_count: u32,
}

impl SegmentManager {
    pub fn from_data_manager(data_mgr: DataManager) -> Self {
        Self {
            data_mgr,
            metadata: Metadata::default(),
            segments: Vec::new(),
            segments_count: 0,
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
        let min_memory_size: u32 =
            (START_INIT_DATA.len() + METADATA_SIZE + END_INIT_DATA.len() + 4) as u32;
        if memory_size < min_memory_size {
            return Err(DataManagerError::InvalidArgument(format!(
                "memory_size must be >= {min_memory_size}"
            )));
        }

        self.segments.clear();
        self.segments_count = 0;
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

        seg_mgr.validate_loaded_metadata()?;
        seg_mgr.load_segments()?;

        Ok(seg_mgr)
    }

    fn validate_loaded_metadata(&mut self) -> Result<(), DataManagerError> {
        let min_memory_size: u32 =
            (START_INIT_DATA.len() + METADATA_SIZE + END_INIT_DATA.len() + 4) as u32;
        if self.metadata.memory_size < min_memory_size {
            return Err(DataManagerError::InvalidArgument(format!(
                "invalid metadata: memory_size must be >= {min_memory_size}"
            )));
        }

        // For backends where we can reliably measure the current storage length, ensure the
        // declared memory size fits to avoid out-of-bounds reads and accidental huge allocations.
        if let Some(len) = self
            .data_mgr
            .byte_len()
            .map_err(DataManagerError::DeviceError)?
        {
            if len < self.metadata.memory_size as u64 {
                return Err(DataManagerError::InvalidArgument(format!(
                    "invalid storage: backing store is {len} bytes, but metadata.memory_size is {} bytes",
                    self.metadata.memory_size
                )));
            }
        }

        Ok(())
    }
}

mod tests {
    #[allow(unused)]
    use super::*;
    use crate::data_mgr::DataInterfaceType;

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

    fn minimal_initialized_storage(meta: Metadata) -> Vec<u8> {
        let mut out = vec![0u8; START_INIT_DATA.len() + METADATA_SIZE + END_INIT_DATA.len()];

        out[..START_INIT_DATA.len()].copy_from_slice(START_INIT_DATA);

        let meta_bytes = meta.pack();
        assert_eq!(meta_bytes.len(), METADATA_SIZE);
        out[START_INIT_DATA.len()..START_INIT_DATA.len() + METADATA_SIZE].copy_from_slice(&meta_bytes);

        let end_off = START_INIT_DATA.len() + METADATA_SIZE;
        out[end_off..end_off + END_INIT_DATA.len()].copy_from_slice(END_INIT_DATA);

        out
    }

    #[test]
    fn try_load_rejects_invalid_metadata_memory_size_too_small() {
        let min_memory_size: u32 =
            (START_INIT_DATA.len() + METADATA_SIZE + END_INIT_DATA.len() + 4) as u32;

        let mut meta = Metadata::default();
        meta.version = ARCHITECTURE_VERSION;
        meta.memory_size = min_memory_size - 1;
        meta.dump_protection = EncryptionLevel::Low;

        let buf = minimal_initialized_storage(meta);
        let err = SegmentManager::try_load(DataInterfaceType::Memory(buf))
            .err()
            .expect("expected error");
        assert!(
            matches!(err, DataManagerError::InvalidArgument(_)),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn try_load_rejects_memory_size_larger_than_backing_store_for_memory_backend() {
        let min_memory_size: u32 =
            (START_INIT_DATA.len() + METADATA_SIZE + END_INIT_DATA.len() + 4) as u32;

        let mut meta = Metadata::default();
        meta.version = ARCHITECTURE_VERSION;
        meta.memory_size = min_memory_size + 128;
        meta.dump_protection = EncryptionLevel::Low;

        let buf = minimal_initialized_storage(meta);
        let err = SegmentManager::try_load(DataInterfaceType::Memory(buf))
            .err()
            .expect("expected error");
        assert!(
            matches!(err, DataManagerError::InvalidArgument(_)),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn try_load_rejects_segments_count_too_large() {
        let mut mgr =
            SegmentManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.init_device(1024, EncryptionLevel::Low).expect("init device");

        let mut dump = mgr.get_dump().expect("dump");

        let segments_count_addr = (mgr.metadata.memory_size - 4) as usize;
        dump[segments_count_addr..segments_count_addr + 4].copy_from_slice(&1_000_001u32.to_le_bytes());

        let err = SegmentManager::try_load(DataInterfaceType::Memory(dump))
            .err()
            .expect("expected error");
        match err {
            DataManagerError::InvalidArgument(msg) => {
                assert!(msg.contains("segments_count too large"), "unexpected msg: {msg}");
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
