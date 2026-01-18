use borsh::{BorshDeserialize, BorshSerialize};
use borsh_derive::{BorshDeserialize, BorshSerialize};

use crate::{
    data_mgr::{DataManagerExt, DeviceError, RecordDirection},
    seg_mgr::{DataError, DataInfo, SegmentError, METADATA_SIZE},
};

use super::{Segment, SegmentManager, ARCHITECTURE_VERSION};

pub const START_INIT_DATA: &[u8] = b"\0<METADATA>\0";
pub const END_INIT_DATA: &[u8] = b"\0</METADATA>\0";
pub const MASTER_PASSWORD_HASH_SIZE: usize = 32;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Metadata {
    pub version: u32,
    pub memory_size: u32,
    pub master_password_hash: [u8; MASTER_PASSWORD_HASH_SIZE],
}

impl Metadata {
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
        self.data_mgr.write_value(seg.meta_address, seg.info)
    }

    pub fn add_segment_meta(&mut self, segment: Segment) -> Result<(), DeviceError> {
        self.save_segment_meta(&segment)?;
        self.segments.insert(0, segment);
        self.save_segments_count()?;
        Ok(())
    }

    pub fn load_segments(&mut self) -> Result<(), DeviceError> {
        let data_infos = self
            .data_mgr
            .read_values::<DataInfo>(self.segments_info_address(), RecordDirection::Left)?;
        self.segments.clear();
        for (i, info) in data_infos.into_iter().rev().enumerate() {
            let meta_address = self.segment_meta_address(i as u32);
            let segment = Segment::new(self.data_mgr.clone(), meta_address, info);
            self.segments.insert(0, segment);
        }
        Ok(())
    }

    pub fn format_data(&mut self) -> Result<(), DeviceError> {
        let data = vec![0u8; self.metadata.memory_size as usize];
        self.data_mgr.write_data(0, &data)
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
