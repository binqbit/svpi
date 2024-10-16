use super::{SegmentManager, END_INIT_DATA, ROOT_PASSWORD_SIZE, SEGMENT_SIZE, START_INIT_DATA};

impl SegmentManager {
    pub fn start_init_data_address(&self) -> u32 {
        0
    }

    pub fn end_init_data_address(&self) -> u32 {
        self.root_password_address() + ROOT_PASSWORD_SIZE
    }

    pub fn version_address(&self) -> u32 {
        START_INIT_DATA.len() as u32
    }

    pub fn memory_size_address(&self) -> u32 {
        self.version_address() + 4
    }

    pub fn root_password_address(&self) -> u32 {
        self.memory_size_address() + 4
    }

    pub fn segments_info_address(&self) -> u32 {
        self.memory_size - 4
    }

    pub fn segment_info_address(&self, index: u32) -> u32 {
        self.segments_info_address() - (index + 1) * SEGMENT_SIZE
    }

    pub fn start_data_address(&self) -> u32 {
        self.end_init_data_address() + END_INIT_DATA.len() as u32
    }

    pub fn end_data_address(&self) -> u32 {
        self.segments_info_address() - self.segments.len() as u32 * SEGMENT_SIZE
    }

    pub fn last_data_address(&self) -> u32 {
        let mut address = self.start_data_address();
        for segment in &self.segments {
            let end_address = segment.address + segment.size;
            if end_address > address {
                address = end_address;
            }
        }
        address
    }
}