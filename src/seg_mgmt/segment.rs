use super::{RawSegmentInfo, Segment, SegmentManager};

impl Segment {
    pub fn new(index: u32, address: u32, size: u32, name: &str) -> Segment {
        let name = name.as_bytes();
        let mut name_buf = [0; 32];
        name_buf[..name.len()].copy_from_slice(name);
        Segment { index, address, size, name: name_buf }
    }

    pub fn from_raw(index: u32, raw: RawSegmentInfo) -> Segment {
        Segment {
            index,
            address: raw.0,
            size: raw.1,
            name: raw.2,
        }
    }

    pub fn to_raw(&self) -> RawSegmentInfo {
        (self.address, self.size, self.name)
    }

    pub fn set_index(&mut self, index: u32) {
        self.index = index;
    }

    pub fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    pub fn set_name(&mut self, name: &str) {
        let name = name.as_bytes();
        self.name = [0; 32];
        self.name[..name.len()].copy_from_slice(name);
    }

    pub fn is_removed(&self) -> bool {
        self.name == [0; 32]
    }

    pub fn get_name(&self) -> String {
        String::from_utf8_lossy(&self.name).trim_end_matches(char::from(0)).to_string()
    }
}

impl SegmentManager {
    pub fn add_segment(&mut self, name: &str, size: u32) -> std::io::Result<Option<&Segment>> {
        let seg_index = self.find_segment_by_name(name).map(|seg| seg.index);
        let address = self.find_new_segment_address(size);
        if let Some(address) = address {
            let segment = Segment::new(self.segments.len() as u32, address, size, name);
            self.save_segment_info(&segment)?;
            self.segments.insert(0, segment);
            self.save_segments_count()?;
            if let Some(seg_index) = seg_index {
                self.remove_segment(seg_index)?;
            }
            Ok(self.segments.first())
        } else {
            Ok(None)
        }
    }

    pub fn remove_segment(&mut self, index: u32) -> std::io::Result<bool> {
        let seg = if let Some(seg) = self.find_segment_by_index(index) {
            seg.name = [0; 32];
            Some(seg.clone())
        } else {
            None
        };
        if let Some(seg) = &seg {
            self.save_segment_info(seg)?;
        }
        Ok(seg.is_some())
    }

    pub fn read_segment_data(&mut self, seg: &Segment) -> std::io::Result<String> {
        let data = self.read_data(seg.address, seg.size)?;
        Ok(String::from_utf8_lossy(&data).trim_end_matches(char::from(0)).to_string())
    }

    pub fn write_segment_data(&mut self, seg: &Segment, data: &str) -> std::io::Result<bool> {
        if data.len() <= seg.size as usize {
            self.write_data(seg.address, data.as_bytes())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn find_segment_by_index(&mut self, index: u32) -> Option<&mut Segment> {
        self.segments.iter_mut().find(|seg| seg.index == index)
    }

    pub fn find_segment_by_name(&mut self, name: &str) -> Option<&mut Segment> {
        self.segments.iter_mut().find(|seg| seg.get_name() == name)
    }
}