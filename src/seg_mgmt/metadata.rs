use super::{
    RawSegmentInfo, RecordDirection, Segment, SegmentManager, ARCHITECTURE_VERSION, END_INIT_DATA,
    START_INIT_DATA,
};

impl SegmentManager {
    pub fn check_init_data(&mut self) -> std::io::Result<bool> {
        let start = self.read_data(self.start_init_data_address(), START_INIT_DATA.len() as u32)?;
        if start != START_INIT_DATA {
            return Ok(false);
        }

        let end = self.read_data(self.end_init_data_address(), END_INIT_DATA.len() as u32)?;
        if end != END_INIT_DATA {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn check_architecture_version(&mut self) -> std::io::Result<bool> {
        let version = self.get_version()?;
        Ok(version == ARCHITECTURE_VERSION)
    }

    pub fn get_version(&mut self) -> std::io::Result<u32> {
        self.read_value(self.version_address())
    }

    pub fn set_version(&mut self, version: u32) -> std::io::Result<()> {
        self.write_value(self.version_address(), version)
    }

    pub fn get_memory_size(&mut self) -> std::io::Result<u32> {
        self.read_value(self.memory_size_address())
    }

    pub fn set_memory_size(&mut self, memory_size: u32) -> std::io::Result<()> {
        self.write_value(self.memory_size_address(), memory_size)
    }

    pub fn get_root_password(&mut self) -> std::io::Result<Vec<u8>> {
        self.read_values(self.root_password_address(), RecordDirection::Right)
    }

    pub fn set_root_password(&mut self, password: &[u8]) -> std::io::Result<()> {
        self.write_values(
            self.root_password_address(),
            password,
            RecordDirection::Right,
        )
    }

    pub fn reset_root_password(&mut self) -> std::io::Result<()> {
        self.set_root_password(&[])
    }

    pub fn is_root_password_set(&mut self) -> std::io::Result<bool> {
        let password = self.get_root_password()?;
        Ok(password.len() > 0)
    }

    pub fn save_segments_count(&mut self) -> std::io::Result<()> {
        self.write_value(self.segments_info_address(), self.segments.len() as u32)
    }

    pub fn save_segment_info(&mut self, seg: &Segment) -> std::io::Result<()> {
        self.write_value(self.segment_info_address(seg.index), seg.to_raw())
    }

    pub fn init_segments(&mut self, memory_size: u32) -> std::io::Result<()> {
        self.version = ARCHITECTURE_VERSION;
        self.memory_size = memory_size;
        self.write_data(self.start_init_data_address(), START_INIT_DATA)?;
        self.write_data(self.end_init_data_address(), END_INIT_DATA)?;
        self.set_version(ARCHITECTURE_VERSION)?;
        self.set_memory_size(memory_size)?;
        self.reset_root_password()?;
        let raw_segments: Vec<RawSegmentInfo> = self.segments.iter().map(Segment::to_raw).collect();
        self.write_values(
            self.segments_info_address(),
            &raw_segments,
            RecordDirection::Left,
        )
    }

    pub fn load_segments(&mut self) -> std::io::Result<()> {
        self.version = self.get_version()?;
        self.memory_size = self.get_memory_size()?;
        let raw_segments = self
            .read_values::<RawSegmentInfo>(self.segments_info_address(), RecordDirection::Left)?;
        self.segments.clear();
        for (i, segment) in raw_segments.into_iter().rev().enumerate() {
            self.segments
                .insert(0, Segment::from_raw(i as u32, segment));
        }
        Ok(())
    }

    pub fn format_data(&mut self) -> std::io::Result<()> {
        let memory_size = self.memory_size as usize;
        let data = vec![0u8; memory_size];
        self.write_data(0, &data)
    }

    pub fn get_segments_info(&self) -> Vec<Segment> {
        self.segments
            .iter()
            .filter(|segment| segment.status)
            .cloned()
            .collect()
    }
}
