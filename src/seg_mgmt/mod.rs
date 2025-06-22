use crate::spdm::SerialPortDataManager;

/// Raw Segment Info: (address, size, data_type, is_encrypted, is_active, name)
pub type RawSegmentInfo = (u32, u32, DataType, bool, bool, [u8; 32]);
pub const SEGMENT_SIZE: u32 = std::mem::size_of::<RawSegmentInfo>() as u32;
pub const START_INIT_DATA: &[u8] = b"\0<METADATA>\0";
pub const END_INIT_DATA: &[u8] = b"\0</METADATA>\0";
pub const ROOT_PASSWORD_SIZE: u32 = 128;
pub const ARCHITECTURE_VERSION: u32 = 5;

pub struct SegmentManager {
    pub spdm: SerialPortDataManager,
    pub version: u32,
    pub memory_size: u32,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub spdm: SerialPortDataManager,
    pub meta_address: u32,
    pub address: u32,
    pub size: u32,
    pub data_type: DataType,
    pub is_encrypted: bool,
    pub is_active: bool,
    pub name: [u8; 32],
}

impl SerialPortDataManager {
    pub fn into_segment_manager(self) -> SegmentManager {
        SegmentManager {
            spdm: self,
            version: ARCHITECTURE_VERSION,
            memory_size: 0,
            segments: Vec::new(),
        }
    }
}

mod addresses;
mod data;
mod mem_mgmt;
mod metadata;
mod segment;

pub use data::*;
pub use segment::*;
