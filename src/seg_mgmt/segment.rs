use super::{DataType, RawSegmentInfo, Segment, SegmentManager};

impl Segment {
    pub fn new(index: u32, address: u32, size: u32, data_type: DataType, name: &str) -> Segment {
        let name = name.as_bytes();
        let mut name_buf = [0; 32];
        name_buf[..name.len()].copy_from_slice(name);
        Segment {
            index,
            address,
            size,
            data_type,
            status: true,
            name: name_buf,
        }
    }

    pub fn from_raw(index: u32, raw: RawSegmentInfo) -> Segment {
        Segment {
            index,
            address: raw.0,
            size: raw.1,
            data_type: raw.2,
            status: raw.3,
            name: raw.4,
        }
    }

    pub fn to_raw(&self) -> RawSegmentInfo {
        (
            self.address,
            self.size,
            self.data_type.clone(),
            self.status,
            self.name,
        )
    }

    pub fn set_index(&mut self, index: u32) {
        self.index = index;
    }

    pub fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    pub fn set_data_type(&mut self, data_type: DataType) {
        self.data_type = data_type;
    }

    pub fn set_status(&mut self, status: bool) {
        self.status = status;
    }

    pub fn set_name(&mut self, name: &str) {
        let name = name.as_bytes();
        self.name = [0; 32];
        self.name[..name.len()].copy_from_slice(name);
    }

    pub fn get_name(&self) -> String {
        String::from_utf8_lossy(&self.name)
            .trim_end_matches(char::from(0))
            .to_string()
    }
}

impl SegmentManager {
    pub fn set_segment(
        &mut self,
        name: &str,
        data: &[u8],
        data_type: DataType,
    ) -> std::io::Result<Option<&Segment>> {
        let seg_index = self.find_segment_by_name(name).map(|seg| seg.index);
        let address = self.find_new_segment_address(data.len() as u32);
        if let Some(address) = address {
            let segment = Segment::new(
                self.segments.len() as u32,
                address,
                data.len() as u32,
                data_type,
                name,
            );
            self.save_segment_info(&segment)?;
            self.write_segment_data(&segment, data)?;
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
            seg.set_status(false);
            Some(seg.clone())
        } else {
            None
        };
        if let Some(seg) = &seg {
            self.save_segment_info(seg)?;
        }
        Ok(seg.is_some())
    }

    pub fn read_segment_data(&mut self, seg: &Segment) -> std::io::Result<Vec<u8>> {
        self.read_data(seg.address, seg.size)
    }

    pub fn write_segment_data(&mut self, seg: &Segment, data: &[u8]) -> std::io::Result<bool> {
        if data.len() <= seg.size as usize {
            self.write_data(seg.address, data)?;
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
