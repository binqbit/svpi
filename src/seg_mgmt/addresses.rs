use super::{SegmentManager, END_INIT_DATA, ROOT_PASSWORD_SIZE, SEGMENT_SIZE, START_INIT_DATA};

impl SegmentManager {
    pub fn start_init_data_address(&self) -> u32 {
        0
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

    pub fn end_init_data_address(&self) -> u32 {
        self.root_password_address() + ROOT_PASSWORD_SIZE
    }

    pub fn start_data_address(&self) -> u32 {
        self.end_init_data_address() + END_INIT_DATA.len() as u32
    }

    pub fn end_data_address(&self) -> u32 {
        self.segments_info_address() - self.segments.len() as u32 * SEGMENT_SIZE
    }

    pub fn segment_meta_address(&self, index: u32) -> u32 {
        self.segments_info_address() - (index + 1) * SEGMENT_SIZE
    }

    pub fn segments_info_address(&self) -> u32 {
        self.memory_size - 4
    }

    pub fn next_data_address(&self) -> u32 {
        if let Some(last_segment) = self.segments.first() {
            return last_segment.address + last_segment.size;
        } else {
            return self.start_data_address();
        }
    }
}
