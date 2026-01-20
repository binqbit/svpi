use crate::seg_mgr::{
    segment::SegmentError, Segment, SegmentManager, DATA_NAME_SIZE, SEGMENT_INFO_SIZE,
};
use crate::{data_mgr::DataManagerExt, data_mgr::DeviceError};

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
            }

            segment.update_meta()?;
            optimized_address += segment.info.size as u32;
        }

        let new_meta_size: u32 = (segments.len() as u32)
            .checked_mul(SEGMENT_INFO_SIZE as u32)
            .ok_or(SegmentError::UpdateInfoError(DeviceError::WriteError))?;
        let new_meta_start = self
            .segments_info_address()
            .checked_sub(new_meta_size)
            .ok_or(SegmentError::UpdateInfoError(DeviceError::WriteError))?;

        if optimized_address < new_meta_start {
            let wipe_len: usize = (new_meta_start - optimized_address)
                .try_into()
                .map_err(|_| SegmentError::UpdateInfoError(DeviceError::WriteError))?;
            self.data_mgr
                .write_zeroes(optimized_address, wipe_len)
                .map_err(SegmentError::UpdateInfoError)?;
        }

        self.segments = segments;
        self.save_segments_count()
            .map_err(SegmentError::UpdateInfoError)?;

        Ok(optimized_size)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        data_mgr::DataInterfaceType,
        seg_mgr::{DataType, EncryptionLevel, SegmentManager, SEGMENT_INFO_SIZE},
    };
    use crate::{data_mgr::DataManagerExt, seg_mgr::DATA_NAME_SIZE};

    fn setup_mgr() -> SegmentManager {
        let mut mgr =
            SegmentManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.init_device(256, EncryptionLevel::Low).expect("device");
        mgr
    }

    #[test]
    fn optimize_moves_segments() {
        let mut mgr = setup_mgr();
        let start = mgr.start_data_address();
        mgr.set_segment("a", b"foo", DataType::Plain, None).unwrap();
        mgr.set_segment("b", b"bar", DataType::Plain, None).unwrap();
        let addr_a = mgr.find_segment_by_name("a").unwrap().info.address;
        let addr_b = mgr.find_segment_by_name("b").unwrap().info.address;
        assert_eq!(addr_a, start);
        assert_eq!(addr_b, start + 3);
        {
            let seg = mgr.find_segment_by_name("a").unwrap();
            let removed_address = seg.info.address;
            let removed_size = seg.info.size as usize;
            let removed_meta = seg.meta_address;
            seg.remove().unwrap();

            let wiped_data = seg
                .data_mgr
                .read_data(removed_address, removed_size)
                .unwrap();
            assert_eq!(wiped_data, vec![0u8; removed_size]);

            let wiped_meta = seg
                .data_mgr
                .read_data(removed_meta, SEGMENT_INFO_SIZE)
                .unwrap();
            assert_eq!(wiped_meta, vec![0u8; SEGMENT_INFO_SIZE]);
        }
        let _optimized = mgr.optimize_segments().unwrap();
        let addr_b_after = mgr.find_segment_by_name("b").unwrap().info.address;
        assert_eq!(addr_b_after, start);
    }

    #[test]
    fn optimize_wipes_old_data_ranges() {
        let mut mgr =
            SegmentManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.init_device(1024, EncryptionLevel::Low).expect("device");

        let start = mgr.start_data_address();

        let a_data = vec![0xAAu8; 40];
        let b_data = vec![0xBBu8; 30];
        let c_data = vec![0xCCu8; 50];

        mgr.set_segment("a", &a_data, DataType::Binary, None)
            .unwrap();
        mgr.set_segment("b", &b_data, DataType::Binary, None)
            .unwrap();
        mgr.set_segment("c", &c_data, DataType::Binary, None)
            .unwrap();

        let c_old_address = mgr.find_segment_by_name("c").unwrap().info.address;
        let c_size = mgr.find_segment_by_name("c").unwrap().info.size;

        {
            let seg = mgr.find_segment_by_name("b").unwrap();
            seg.remove().unwrap();
        }

        mgr.optimize_segments().unwrap();

        let (c_new_address, read_c) = {
            let c = mgr.find_segment_by_name("c").unwrap();
            (c.info.address, c.read_data().unwrap().to_bytes().unwrap())
        };
        assert_eq!(c_new_address, start + a_data.len() as u32);
        assert_eq!(read_c, c_data);

        let optimized_end = start + (a_data.len() + c_data.len()) as u32;
        let new_meta_start =
            mgr.segments_info_address() - (mgr.segments.len() as u32 * SEGMENT_INFO_SIZE as u32);

        let wipe_len = (new_meta_start - optimized_end) as usize;
        let wiped = mgr.data_mgr.read_data(optimized_end, wipe_len).unwrap();
        assert_eq!(wiped, vec![0u8; wipe_len]);

        // The old tail (after the new end) is wiped.
        let c_old_end = c_old_address + c_size;
        let old_tail_len = (c_old_end - optimized_end) as usize;
        let old_tail = mgr.data_mgr.read_data(optimized_end, old_tail_len).unwrap();
        assert_eq!(old_tail, vec![0u8; old_tail_len]);

        // Ensure no active segment has a blank name after optimize (sanity).
        assert!(mgr
            .segments
            .iter()
            .all(|s| s.info.name != [0u8; DATA_NAME_SIZE]));
    }
}
