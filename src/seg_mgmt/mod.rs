use crate::spdm::SerialPortDataManager;

pub type RawSegmentInfo = (u32, u32, [u8; 32]);
pub const SEGMENT_SIZE: u32 = std::mem::size_of::<RawSegmentInfo>() as u32;
pub const START_INIT_DATA: &[u8] = b"\0<INIT_SEGMENTS_DATA>\0";
pub const END_INIT_DATA: &[u8] = b"\0</INIT_SEGMENTS_DATA>\0";

pub struct SegmentManager {
    spdm: SerialPortDataManager,
    pub memory_size: u32,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub index: u32,
    pub address: u32,
    pub size: u32,
    pub name: [u8; 32],
}

impl SerialPortDataManager {
    pub fn into_segment_manager(self) -> SegmentManager {
        SegmentManager {
            spdm: self,
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