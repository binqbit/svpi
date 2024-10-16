use super::{RawSegmentInfo, RecordDirection, Segment, SegmentManager, END_INIT_DATA, START_INIT_DATA};

impl SegmentManager {
    pub fn get_memory_size(&mut self) -> std::io::Result<Option<u32>> {
        let start = self.read_data(0, START_INIT_DATA.len() as u32)?;
        if start != START_INIT_DATA {
            return Ok(None);
        }

        let end = self.read_data(START_INIT_DATA.len() as u32 + 4, END_INIT_DATA.len() as u32)?;
        if end != END_INIT_DATA {
            return Ok(None);
        }

        let address = self.read_value::<u32>(START_INIT_DATA.len() as u32)?;
        Ok(Some(address))
    }

    pub fn get_root_password(&mut self) -> std::io::Result<Vec<u8>> {
        self.read_values(self.root_password_address(), RecordDirection::Right)
    }

    pub fn set_root_password(&mut self, password: &[u8]) -> std::io::Result<()> {
        self.write_values(self.root_password_address(), password, RecordDirection::Right)
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
        self.memory_size = memory_size;
        self.write_data(0, START_INIT_DATA)?;
        self.write_value(START_INIT_DATA.len() as u32, self.memory_size)?;
        self.write_data(START_INIT_DATA.len() as u32 + 4, END_INIT_DATA)?;
        self.reset_root_password()?;
        let raw_segments: Vec<RawSegmentInfo> = self.segments.iter().map(Segment::to_raw).collect();
        self.write_values(self.segments_info_address(), &raw_segments, RecordDirection::Left)
    }

    pub fn load_segments(&mut self) -> std::io::Result<bool> {
        if let Some(memory_size) = self.get_memory_size()? {
            self.memory_size = memory_size;
            let raw_segments = self.read_values::<RawSegmentInfo>(self.segments_info_address(), RecordDirection::Left)?;
            self.segments.clear();
            for (i, segment) in raw_segments.into_iter().rev().enumerate() {
                self.segments.insert(0, Segment::from_raw(i as u32, segment));
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn format_data(&mut self) -> std::io::Result<()> {
        let memory_size = self.memory_size as usize;
        let data = vec![0u8; memory_size];
        self.write_data(0, &data)
    }

    pub fn get_segments_info(&self) -> Vec<Segment> {
        self.segments.iter()
            .filter(|segment| segment.status)
            .cloned()
            .collect()
    }
}