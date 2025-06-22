use crate::seg_mgmt::segment::SegmentError;

use super::{Segment, SegmentManager, SEGMENT_SIZE};

impl SegmentManager {
    pub fn filter_and_sort_segments(&self) -> Vec<Segment> {
        let mut segments: Vec<Segment> = self.get_active_segments().into_iter().cloned().collect();
        segments.sort_by(|a, b| b.address.cmp(&a.address));
        for (i, segment) in segments.iter_mut().rev().enumerate() {
            segment.meta_address = self.segment_meta_address(i as u32);
        }
        segments
    }

    pub fn find_new_segment_address(&self, size: u32) -> Option<u32> {
        let next_data_address = self.next_data_address();
        let end_data_address = self.end_data_address();

        if next_data_address + size > end_data_address - SEGMENT_SIZE {
            return None;
        } else {
            return Some(next_data_address);
        }
    }

    pub fn free_memory_size(&self) -> u32 {
        self.end_data_address() - self.next_data_address()
    }

    pub fn memory_to_optimize(&self) -> u32 {
        self.segments
            .iter()
            .filter(|segment| !segment.is_active)
            .map(|segment| segment.size + SEGMENT_SIZE)
            .sum::<u32>()
    }

    pub fn optimize_segments(&mut self) -> Result<u32, SegmentError> {
        let optimized_size = self.memory_to_optimize();
        let mut segments = self.filter_and_sort_segments();

        let mut optimized_address = self.start_data_address();
        for segment in segments.iter_mut().rev() {
            if optimized_address != segment.address {
                let data = segment
                    .read_data(None)?
                    .to_bytes(None)
                    .map_err(SegmentError::DataError)?;
                segment.set_address(optimized_address);
                segment.write_data(&data)?;
                segment.update_meta()?;
            }
            optimized_address += segment.size;
        }
        if optimized_size > 0 {
            self.segments = segments;
            self.save_segments_count()
                .map_err(SegmentError::UpdateInfoError)?;
        }

        Ok(optimized_size)
    }
}
