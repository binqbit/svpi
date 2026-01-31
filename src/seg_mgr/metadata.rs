use borsh::{BorshDeserialize, BorshSerialize};
use borsh_derive::{BorshDeserialize, BorshSerialize};

use crate::{
    data_mgr::{DataManagerExt, DeviceError},
    seg_mgr::{DataError, DataInfo, SegmentError, METADATA_SIZE, SEGMENT_INFO_SIZE},
};

use super::{EncryptionLevel, Segment, SegmentManager, ARCHITECTURE_VERSION};

pub const START_INIT_DATA: &[u8] = b"\0<METADATA>\0";
pub const END_INIT_DATA: &[u8] = b"\0</METADATA>\0";
pub const MASTER_PASSWORD_HASH_SIZE: usize = 32;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Metadata {
    pub version: u32,
    pub memory_size: u32,
    pub dump_protection: EncryptionLevel,
    pub master_password_hash: [u8; MASTER_PASSWORD_HASH_SIZE],
}

impl Metadata {
    pub const SIZE: usize = 4 + 4 + 1 + MASTER_PASSWORD_HASH_SIZE;

    pub fn pack(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.serialize(&mut buffer)
            .expect("Failed to serialize metadata");
        buffer
    }

    pub fn unpack(data: &[u8]) -> Result<Self, DataError> {
        Metadata::try_from_slice(data).map_err(|_| DataError::UnpackError)
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            version: ARCHITECTURE_VERSION,
            memory_size: 0,
            dump_protection: EncryptionLevel::Medium,
            master_password_hash: [0; MASTER_PASSWORD_HASH_SIZE],
        }
    }
}

impl SegmentManager {
    pub fn read_architecture_version(&mut self) -> Result<u32, DeviceError> {
        self.data_mgr.read_value::<u32>(self.version_address())
    }

    pub fn init_metadata(&mut self) -> Result<(), DeviceError> {
        self.data_mgr
            .init_memory(self.metadata.memory_size as usize)?;
        self.format_data()?;
        self.data_mgr
            .write_data(self.start_init_data_address(), START_INIT_DATA)?;
        self.data_mgr
            .write_data(self.end_init_data_address(), END_INIT_DATA)?;
        self.save_metadata()?;
        self.save_segments_count()?;
        Ok(())
    }

    pub fn save_metadata(&mut self) -> Result<(), DeviceError> {
        let data = self.metadata.pack();
        self.data_mgr.write_data(self.metadata_address(), &data)?;
        Ok(())
    }

    pub fn load_metadata(&mut self) -> Result<(), SegmentError> {
        let data = self
            .data_mgr
            .read_data(self.metadata_address(), METADATA_SIZE)
            .map_err(SegmentError::ReadError)?;
        self.metadata = Metadata::unpack(&data).map_err(SegmentError::DataError)?;
        Ok(())
    }

    pub fn set_master_password_hash(
        &mut self,
        hash: [u8; MASTER_PASSWORD_HASH_SIZE],
    ) -> Result<(), DeviceError> {
        self.metadata.master_password_hash = hash;
        self.save_metadata()
    }

    pub fn reset_master_password_hash(&mut self) -> Result<(), DeviceError> {
        self.metadata.master_password_hash = [0; MASTER_PASSWORD_HASH_SIZE];
        self.save_metadata()
    }

    pub fn save_segments_count(&mut self) -> Result<(), DeviceError> {
        self.data_mgr
            .write_value(self.segments_info_address(), self.segments.len() as u32)
    }

    pub fn save_segment_meta(&mut self, seg: &Segment) -> Result<(), DeviceError> {
        let data = seg.info.pack();
        self.data_mgr.write_data(seg.meta_address, &data)
    }

    pub fn add_segment_meta(&mut self, segment: Segment) -> Result<(), DeviceError> {
        self.save_segment_meta(&segment)?;
        self.segments.insert(0, segment);
        self.save_segments_count()?;
        Ok(())
    }

    pub fn load_segments(&mut self) -> Result<(), DeviceError> {
        self.segments.clear();

        let count = self
            .data_mgr
            .read_value::<u32>(self.segments_info_address())? as usize;
        if count == 0 {
            return Ok(());
        }

        let total_size = count
            .checked_mul(SEGMENT_INFO_SIZE)
            .ok_or(DeviceError::ReadError)?;
        let start_address = self
            .segments_info_address()
            .checked_sub(total_size as u32)
            .ok_or(DeviceError::ReadError)?;

        let data = self.data_mgr.read_data(start_address, total_size)?;
        if data.len() < total_size {
            return Err(DeviceError::ReadError);
        }

        for (i, chunk) in data.chunks_exact(SEGMENT_INFO_SIZE).enumerate() {
            let info = DataInfo::unpack(chunk).map_err(|_| DeviceError::ReadError)?;
            let meta_address = start_address + (i as u32) * SEGMENT_INFO_SIZE as u32;
            let segment = Segment::new(self.data_mgr.clone(), meta_address, info);
            self.segments.push(segment);
        }
        Ok(())
    }

    pub fn format_data(&mut self) -> Result<(), DeviceError> {
        self.data_mgr
            .write_zeroes(0, self.metadata.memory_size as usize)
    }

    pub fn get_active_segments<'a>(&'a self) -> Vec<&'a Segment> {
        self.segments
            .iter()
            .filter(|segment| segment.is_active())
            .collect()
    }

    pub fn get_active_segments_mut<'a>(&'a mut self) -> Vec<&'a mut Segment> {
        self.segments
            .iter_mut()
            .filter(|segment| segment.is_active())
            .collect::<Vec<&mut Segment>>()
    }

    pub fn get_dump(&mut self) -> Result<Vec<u8>, DeviceError> {
        let start_address = self.start_init_data_address();
        self.data_mgr
            .read_data(start_address, self.metadata.memory_size as usize)
    }

    pub fn set_dump(&mut self, data: &[u8]) -> Result<(), DeviceError> {
        let start_address = self.start_init_data_address();
        self.data_mgr.write_data(start_address, data)
    }

    pub fn check_init_data(&mut self) -> Result<bool, DeviceError> {
        let start = self
            .data_mgr
            .read_data(self.start_init_data_address(), START_INIT_DATA.len())?;
        if start != START_INIT_DATA {
            return Ok(false);
        }

        let end = self
            .data_mgr
            .read_data(self.end_init_data_address(), END_INIT_DATA.len())?;
        if end != END_INIT_DATA {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn check_architecture_version(&mut self) -> Result<bool, DeviceError> {
        let version = self.read_architecture_version()?;
        Ok(version == ARCHITECTURE_VERSION)
    }

    pub fn check_master_password_hash(&self, hash: [u8; MASTER_PASSWORD_HASH_SIZE]) -> bool {
        self.metadata.master_password_hash == hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_mgr::{DataInterfaceType, DataManagerExt};
    use crate::seg_mgr::DataType;

    #[test]
    fn init_metadata_writes_markers_and_metadata() {
        let mut mgr =
            SegmentManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.init_device(512, EncryptionLevel::Low)
            .expect("init device");

        let mut data_mgr = mgr.data_mgr.clone();

        let start = data_mgr
            .read_data(mgr.start_init_data_address(), START_INIT_DATA.len())
            .unwrap();
        assert_eq!(start, START_INIT_DATA);

        let end = data_mgr
            .read_data(mgr.end_init_data_address(), END_INIT_DATA.len())
            .unwrap();
        assert_eq!(end, END_INIT_DATA);

        let version = mgr.read_architecture_version().unwrap();
        assert_eq!(version, ARCHITECTURE_VERSION);

        let count = data_mgr
            .read_value::<u32>(mgr.segments_info_address())
            .unwrap();
        assert_eq!(count, 0);

        mgr.load_metadata().expect("load metadata");
        assert_eq!(mgr.metadata.memory_size, 512);
        assert_eq!(mgr.metadata.dump_protection, EncryptionLevel::Low);
    }

    #[test]
    fn try_load_roundtrip_after_init() {
        let mut mgr =
            SegmentManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.init_device(1024, EncryptionLevel::Medium)
            .expect("init device");
        mgr.set_segment("a", b"hi", DataType::Plain, None)
            .expect("set segment");

        let dump = mgr.get_dump().expect("dump");

        let mut loaded = SegmentManager::try_load(DataInterfaceType::Memory(dump)).expect("load");
        assert_eq!(loaded.metadata.memory_size, 1024);
        assert_eq!(loaded.metadata.dump_protection, EncryptionLevel::Medium);
        assert!(loaded.find_segment_by_name("a").is_some());
    }
}
