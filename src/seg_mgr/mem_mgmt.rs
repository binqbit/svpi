use crate::data_mgr::{DataManagerExt, DeviceError};
use crate::seg_mgr::{
    segment::SegmentError, DataManagerError, Segment, SegmentManager, DATA_NAME_SIZE,
    SEGMENT_INFO_SIZE,
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
            None
        } else {
            Some(next_data_address)
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

    pub fn resize_memory(&mut self, memory_size: Option<u32>) -> Result<usize, DataManagerError> {
        let old_memory_size = self.metadata.memory_size;
        let start_data_address = self.start_data_address();

        let active_segments = self.get_active_segments();
        let active_meta_bytes: usize = active_segments
            .len()
            .checked_mul(SEGMENT_INFO_SIZE)
            .ok_or_else(|| DataManagerError::InvalidArgument("meta size overflow".to_string()))?;
        let active_payload_bytes: u64 = active_segments
            .iter()
            .map(|segment| segment.info.size as u64)
            .sum();

        let min_memory_size: u32 = (start_data_address as u64)
            .checked_add(active_payload_bytes)
            .and_then(|v| v.checked_add(active_meta_bytes as u64))
            .and_then(|v| v.checked_add(4))
            .and_then(|v| u32::try_from(v).ok())
            .ok_or_else(|| DataManagerError::InvalidArgument("memory_size overflow".to_string()))?;

        let new_memory_size = memory_size.unwrap_or(min_memory_size);
        if new_memory_size < min_memory_size {
            return Err(DataManagerError::InvalidArgument(format!(
                "memory_size must be >= {min_memory_size}"
            )));
        }

        let optimized_bytes = self.optimize_segments().map_err(|err| match err {
            SegmentError::UpdateInfoError(e)
            | SegmentError::WriteError(e)
            | SegmentError::ReadError(e) => DataManagerError::DeviceError(e),
            _ => DataManagerError::InvalidArgument(format!("optimize failed: {err}")),
        })?;

        let mut segments: Vec<Segment> = self.get_active_segments().into_iter().cloned().collect();
        segments.sort_by(|a, b| b.info.address.cmp(&a.info.address));

        let segments_count: u32 = segments.len().try_into().map_err(|_| {
            DataManagerError::InvalidArgument("segments_count overflow".to_string())
        })?;

        let meta_bytes: usize = segments
            .len()
            .checked_mul(SEGMENT_INFO_SIZE)
            .ok_or_else(|| DataManagerError::InvalidArgument("meta size overflow".to_string()))?;
        let meta_bytes_u32: u32 = meta_bytes
            .try_into()
            .map_err(|_| DataManagerError::InvalidArgument("meta size overflow".to_string()))?;

        let payload_end_current: u32 = segments
            .iter()
            .map(|seg| seg.info.address.saturating_add(seg.info.size))
            .max()
            .unwrap_or(start_data_address);

        let new_meta_start = new_memory_size
            .checked_sub(4)
            .and_then(|v| v.checked_sub(meta_bytes_u32))
            .ok_or_else(|| {
                DataManagerError::InvalidArgument("memory_size too small".to_string())
            })?;
        if payload_end_current > new_meta_start {
            let required = (payload_end_current as u64)
                .checked_add(meta_bytes as u64)
                .and_then(|v| v.checked_add(4))
                .and_then(|v| u32::try_from(v).ok())
                .unwrap_or(u32::MAX);
            return Err(DataManagerError::InvalidArgument(format!(
                "memory_size must be >= {required} for current data layout"
            )));
        }

        if new_memory_size > old_memory_size {
            self.data_mgr
                .init_memory(new_memory_size as usize)
                .map_err(DataManagerError::DeviceError)?;
        }

        if meta_bytes != 0 {
            let mut meta_buf = Vec::with_capacity(meta_bytes);
            for seg in segments.iter() {
                meta_buf.extend_from_slice(&seg.info.pack());
            }
            self.data_mgr
                .write_data(new_meta_start, &meta_buf)
                .map_err(DataManagerError::DeviceError)?;
        }

        self.data_mgr
            .write_value::<u32>(new_memory_size - 4, segments_count)
            .map_err(DataManagerError::DeviceError)?;

        let mut meta_addr = new_meta_start;
        for seg in segments.iter_mut() {
            seg.meta_address = meta_addr;
            meta_addr = meta_addr
                .checked_add(SEGMENT_INFO_SIZE as u32)
                .ok_or_else(|| {
                    DataManagerError::InvalidArgument("meta address overflow".to_string())
                })?;
        }

        self.metadata.memory_size = new_memory_size;
        self.save_metadata()
            .map_err(DataManagerError::DeviceError)?;
        self.segments = segments;

        if new_memory_size > old_memory_size {
            let old_meta_start = old_memory_size
                .checked_sub(4)
                .and_then(|v| v.checked_sub(meta_bytes_u32))
                .ok_or_else(|| {
                    DataManagerError::InvalidArgument("meta address underflow".to_string())
                })?;
            let old_meta_end = old_memory_size;

            let wipe_end = std::cmp::min(old_meta_end, new_meta_start);
            if wipe_end > old_meta_start {
                let wipe_len: usize = (wipe_end - old_meta_start).try_into().map_err(|_| {
                    DataManagerError::InvalidArgument("meta wipe overflow".to_string())
                })?;
                self.data_mgr
                    .write_zeroes(old_meta_start, wipe_len)
                    .map_err(DataManagerError::DeviceError)?;
            }

            if new_meta_start > old_meta_end {
                let wipe_len: usize = (new_meta_start - old_meta_end).try_into().map_err(|_| {
                    DataManagerError::InvalidArgument("meta wipe overflow".to_string())
                })?;
                self.data_mgr
                    .write_zeroes(old_meta_end, wipe_len)
                    .map_err(DataManagerError::DeviceError)?;
            }
        } else if new_memory_size < old_memory_size {
            let tail_len: usize = (old_memory_size - new_memory_size)
                .try_into()
                .map_err(|_| DataManagerError::InvalidArgument("tail wipe overflow".to_string()))?;
            self.data_mgr
                .write_zeroes(new_memory_size, tail_len)
                .map_err(DataManagerError::DeviceError)?;
            self.data_mgr
                .init_memory(new_memory_size as usize)
                .map_err(DataManagerError::DeviceError)?;
        }

        Ok(optimized_bytes)
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

        let c_old_end = c_old_address + c_size;
        let old_tail_len = (c_old_end - optimized_end) as usize;
        let old_tail = mgr.data_mgr.read_data(optimized_end, old_tail_len).unwrap();
        assert_eq!(old_tail, vec![0u8; old_tail_len]);

        assert!(mgr
            .segments
            .iter()
            .all(|s| s.info.name != [0u8; DATA_NAME_SIZE]));
    }

    #[test]
    fn resize_to_packed_min_shrinks_and_preserves_data() {
        let mut mgr =
            SegmentManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.init_device(2048, EncryptionLevel::Low).expect("device");

        let a_data = b"aaaaaaaaaa".to_vec();
        let b_data = vec![0xBBu8; 25];
        let c_data = b"ccc".to_vec();

        mgr.set_segment("a", &a_data, DataType::Binary, None)
            .unwrap();
        mgr.set_segment("b", &b_data, DataType::Binary, None)
            .unwrap();
        mgr.set_segment("c", &c_data, DataType::Binary, None)
            .unwrap();

        {
            let seg = mgr.find_segment_by_name("a").unwrap();
            seg.remove().unwrap();
        }

        mgr.optimize_segments().unwrap();
        let payload_bytes: u64 = mgr.segments.iter().map(|s| s.info.size as u64).sum();
        let meta_bytes: u64 = mgr.segments.len() as u64 * SEGMENT_INFO_SIZE as u64;
        let min_size =
            u32::try_from((mgr.start_data_address() as u64) + payload_bytes + meta_bytes + 4)
                .unwrap();

        mgr.resize_memory(None).unwrap();
        assert_eq!(mgr.metadata.memory_size, min_size);

        let mut dm = mgr.data_mgr.clone();
        assert!(dm.read_data(min_size, 1).is_err());

        {
            let seg = mgr.find_segment_by_name("b").unwrap();
            let read = seg.read_data().unwrap().to_bytes().unwrap();
            assert_eq!(read, b_data);
        }
        {
            let seg = mgr.find_segment_by_name("c").unwrap();
            let read = seg.read_data().unwrap().to_bytes().unwrap();
            assert_eq!(read, c_data);
        }

        let dump = mgr.get_dump().unwrap();
        let mut loaded = SegmentManager::try_load(DataInterfaceType::Memory(dump)).unwrap();
        assert_eq!(loaded.metadata.memory_size, min_size);
        assert!(loaded.find_segment_by_name("b").is_some());
        assert!(loaded.find_segment_by_name("c").is_some());
    }

    #[test]
    fn resize_rejects_too_small_without_mutation() {
        let mut mgr =
            SegmentManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.init_device(1024, EncryptionLevel::Low).expect("device");

        mgr.set_segment("a", b"11111", DataType::Plain, None)
            .unwrap();
        mgr.set_segment("b", b"22222", DataType::Plain, None)
            .unwrap();

        {
            let seg = mgr.find_segment_by_name("a").unwrap();
            seg.remove().unwrap();
        }

        let before_dump = mgr.get_dump().unwrap();

        let err = mgr.resize_memory(Some(64)).unwrap_err();
        assert!(
            err.to_string().contains("memory_size must be >="),
            "unexpected error: {err}"
        );

        let after_dump = mgr.get_dump().unwrap();
        assert_eq!(after_dump, before_dump);

        let read = mgr
            .find_segment_by_name("b")
            .unwrap()
            .read_data()
            .unwrap()
            .to_bytes()
            .unwrap();
        assert_eq!(read, b"22222");
    }

    #[test]
    fn resize_grow_wipes_old_metadata_table() {
        let mut mgr =
            SegmentManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.init_device(512, EncryptionLevel::Low).expect("device");

        mgr.set_segment("a", b"foo", DataType::Plain, None).unwrap();
        mgr.set_segment("b", b"bar", DataType::Plain, None).unwrap();

        mgr.optimize_segments().unwrap();

        let old_size = mgr.metadata.memory_size;
        let meta_bytes = mgr.segments.len() * SEGMENT_INFO_SIZE;
        let old_meta_start = old_size - 4 - meta_bytes as u32;

        {
            let mut dm = mgr.data_mgr.clone();
            let buf = dm.read_data(old_meta_start, meta_bytes + 4).unwrap();
            assert!(buf.iter().any(|b| *b != 0));
        }

        mgr.resize_memory(Some(1024)).unwrap();
        assert_eq!(mgr.metadata.memory_size, 1024);
        let new_meta_start = 1024 - 4 - meta_bytes as u32;

        {
            let mut dm = mgr.data_mgr.clone();
            let wiped = dm.read_data(old_meta_start, meta_bytes + 4).unwrap();
            assert_eq!(wiped, vec![0u8; meta_bytes + 4]);
        }

        if new_meta_start > old_size {
            let mut dm = mgr.data_mgr.clone();
            let gap_len = (new_meta_start - old_size) as usize;
            let wiped = dm.read_data(old_size, gap_len).unwrap();
            assert_eq!(wiped, vec![0u8; gap_len]);
        }

        let dump = mgr.get_dump().unwrap();
        let mut loaded = SegmentManager::try_load(DataInterfaceType::Memory(dump)).unwrap();
        assert_eq!(loaded.metadata.memory_size, 1024);
        assert!(loaded.find_segment_by_name("a").is_some());
        assert!(loaded.find_segment_by_name("b").is_some());
    }

    #[test]
    fn resize_grow_small_does_not_wipe_new_table_overlap() {
        let mut mgr =
            SegmentManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.init_device(512, EncryptionLevel::Low).expect("device");

        mgr.set_segment("a", b"foo", DataType::Plain, None).unwrap();
        mgr.set_segment("b", b"bar", DataType::Plain, None).unwrap();
        mgr.optimize_segments().unwrap();

        let old_size = mgr.metadata.memory_size;
        let meta_bytes = mgr.segments.len() * SEGMENT_INFO_SIZE;
        let old_meta_start = old_size - 4 - meta_bytes as u32;

        let new_size = old_size + 80;
        let new_meta_start = new_size - 4 - meta_bytes as u32;
        assert!(new_meta_start < old_size);

        mgr.resize_memory(Some(new_size)).unwrap();
        assert_eq!(mgr.metadata.memory_size, new_size);

        {
            let mut dm = mgr.data_mgr.clone();
            let wiped_len = (new_meta_start - old_meta_start) as usize;
            let wiped = dm.read_data(old_meta_start, wiped_len).unwrap();
            assert_eq!(wiped, vec![0u8; wiped_len]);
        }

        {
            let mut dm = mgr.data_mgr.clone();
            let overlap_len = (old_size - new_meta_start) as usize;
            let buf = dm.read_data(new_meta_start, overlap_len).unwrap();
            assert!(buf.iter().any(|b| *b != 0));
        }

        let dump = mgr.get_dump().unwrap();
        let mut loaded = SegmentManager::try_load(DataInterfaceType::Memory(dump)).unwrap();
        assert_eq!(loaded.metadata.memory_size, new_size);
        assert!(loaded.find_segment_by_name("a").is_some());
        assert!(loaded.find_segment_by_name("b").is_some());
    }
}
