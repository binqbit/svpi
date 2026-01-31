use crate::seg_mgr::{
    metadata::{END_INIT_DATA, START_INIT_DATA},
    SegmentManager, METADATA_SIZE, SEGMENT_INFO_SIZE,
};

impl SegmentManager {
    pub fn start_init_data_address(&self) -> u32 {
        0
    }

    pub fn metadata_address(&self) -> u32 {
        self.start_init_data_address() + START_INIT_DATA.len() as u32
    }

    pub fn version_address(&self) -> u32 {
        self.metadata_address()
    }

    pub fn end_init_data_address(&self) -> u32 {
        self.metadata_address() + METADATA_SIZE as u32
    }

    pub fn start_data_address(&self) -> u32 {
        self.end_init_data_address() + END_INIT_DATA.len() as u32
    }

    pub fn end_data_address(&self) -> u32 {
        self.segments_info_address() - self.segments.len() as u32 * SEGMENT_INFO_SIZE as u32
    }

    pub fn segment_meta_address(&self, index: u32) -> u32 {
        self.segments_info_address() - (index + 1) * SEGMENT_INFO_SIZE as u32
    }

    pub fn segments_info_address(&self) -> u32 {
        self.metadata.memory_size as u32 - 4
    }

    pub fn next_data_address(&self) -> u32 {
        let last_address = self
            .segments
            .iter()
            .filter(|s| s.is_active())
            .map(|s| s.info.address + s.info.size as u32)
            .max();
        if let Some(last_segment) = last_address {
            return last_segment;
        } else {
            return self.start_data_address();
        }
    }
}
