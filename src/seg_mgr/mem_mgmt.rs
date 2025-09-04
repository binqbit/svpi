use crate::seg_mgr::{
    segment::SegmentError, Segment, SegmentManager, DATA_NAME_SIZE, SEGMENT_INFO_SIZE,
};

impl SegmentManager {
    pub fn filter_and_sort_segments(&self) -> Vec<Segment> {
        let mut segments: Vec<Segment> = self.get_active_segments().into_iter().cloned().collect();
        segments.sort_by(|a, b| b.info.address.cmp(&a.info.address));
        for (i, segment) in segments.iter_mut().rev().enumerate() {
            segment.meta_address = self.segment_meta_address(i as u32);
        }
        segments
    }

    pub fn find_new_segment_address(&self, size: u32) -> Option<u32> {
        let next_data_address = self.next_data_address();
        let end_data_address = self.end_data_address();

        if next_data_address + size > end_data_address - SEGMENT_INFO_SIZE as u32 {
            return None;
        } else {
            return Some(next_data_address);
        }
    }

    pub fn free_memory_size(&self) -> usize {
        (self.end_data_address() - self.next_data_address()) as usize
    }

    pub fn memory_to_optimize(&self) -> usize {
        self.segments
            .iter()
            .filter(|segment| segment.info.name != [0; DATA_NAME_SIZE])
            .map(|segment| segment.info.size as usize + SEGMENT_INFO_SIZE)
            .sum::<usize>()
    }

    pub fn optimize_segments(&mut self) -> Result<usize, SegmentError> {
        let optimized_size = self.memory_to_optimize();
        let mut segments = self.filter_and_sort_segments();

        let mut optimized_address = self.start_data_address();
        for segment in segments.iter_mut().rev() {
            if optimized_address != segment.info.address {
                let data = segment
                    .read_data()?
                    .to_bytes()
                    .map_err(SegmentError::DataError)?;
                segment.set_address(optimized_address);
                segment.write_data(&data)?;
                segment.update_meta()?;
            }
            optimized_address += segment.info.size as u32;
        }
        if optimized_size > 0 {
            self.segments = segments;
            self.save_segments_count()
                .map_err(SegmentError::UpdateInfoError)?;
        }

        Ok(optimized_size)
    }
}
