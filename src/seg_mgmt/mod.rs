use crate::spdm::SerialPortDataManager;

pub type RawSegmentInfo = (u32, u32, DataType, bool, [u8; 32]);
pub const SEGMENT_SIZE: u32 = std::mem::size_of::<RawSegmentInfo>() as u32;
pub const START_INIT_DATA: &[u8] = b"\0<INIT_SEGMENTS_DATA>\0";
pub const END_INIT_DATA: &[u8] = b"\0</INIT_SEGMENTS_DATA>\0";
pub const ROOT_PASSWORD_SIZE: u32 = 128;
pub const ARCHITECTURE_VERSION: u32 = 3;

pub struct SegmentManager {
    pub spdm: SerialPortDataManager,
    pub version: u32,
    pub memory_size: u32,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub index: u32,
    pub address: u32,
    pub size: u32,
    pub data_type: DataType,
    pub status: bool,
    pub name: [u8; 32],
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Plain,
    Encrypted,
}

impl SerialPortDataManager {
    pub fn into_segment_manager(self) -> SegmentManager {
        SegmentManager {
            spdm: self,
            version: 0,
            memory_size: 0,
            segments: Vec::new(),
        }
    }
}

mod types;
mod addresses;
mod metadata;
mod segment;
mod mem_mgmt;

pub use types::*;
