use thiserror::Error;

use crate::{
    data_mgr::{DataManagerExt, DeviceError},
    seg_mgr::{
        Data, DataError, DataInfo, DataManager, DataType, SegmentManager, DATA_FINGERPRINT_SIZE,
        DATA_NAME_SIZE, SEGMENT_INFO_SIZE,
    },
};

#[derive(Debug, Error)]
pub enum SegmentError {
    #[error("Data size too large")]
    DataSizeTooLarge,
    #[error("Data not found: {0}")]
    NotFound(String),
    #[error("Data error: {0}")]
    DataError(DataError),
    #[error("Update info error: {0}")]
    UpdateInfoError(DeviceError),
    #[error("Write error: {0}")]
    WriteError(DeviceError),
    #[error("Read error: {0}")]
    ReadError(DeviceError),
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub data_mgr: DataManager,
    pub meta_address: u32,
    pub info: DataInfo,
}

impl Segment {
    pub fn new(data_mgr: DataManager, meta_address: u32, info: DataInfo) -> Segment {
        Segment {
            data_mgr,
            meta_address,
            info,
        }
    }

    pub fn is_active(&self) -> bool {
        self.info.name != [0; DATA_NAME_SIZE]
    }

    pub fn set_name(&mut self, name: &str) {
        self.info.name = [0; DATA_NAME_SIZE];
        let name_len = name.len().min(DATA_NAME_SIZE);
        self.info.name[..name_len].copy_from_slice(name.as_bytes());
    }

    pub fn set_address(&mut self, address: u32) {
        self.info.address = address;
    }

    pub fn get_name(&self) -> String {
        if self.is_active() {
            String::from_utf8_lossy(&self.info.name)
                .trim_end_matches(char::from(0))
                .to_string()
        } else {
            String::from("<deleted>")
        }
    }
}

impl SegmentManager {
    pub fn set_segment<'a>(
        &'a mut self,
        name: &str,
        data: &[u8],
        data_type: DataType,
        password_fingerprint: Option<[u8; DATA_FINGERPRINT_SIZE]>,
    ) -> Result<Option<&'a Segment>, SegmentError> {
        let address = if let Some(address) = self.find_new_segment_address(data.len() as u32) {
            address
        } else {
            return Ok(None);
        };
        let meta_address = self.segment_meta_address(self.segments.len() as u32);
        let info = DataInfo::new(
            name,
            address,
            data,
            data_type,
            password_fingerprint,
            &self
                .segments
                .iter()
                .map(|s| s.info.fingerprint)
                .collect::<Vec<_>>(),
        );

        if let Some(old_seg) = self.find_segment_by_name(name) {
            old_seg.remove()?;
        }

        let mut segment = Segment::new(self.data_mgr.clone(), meta_address, info);
        segment.write_data(&data)?;
        self.add_segment_meta(segment)
            .map_err(SegmentError::UpdateInfoError)?;

        Ok(self.segments.first())
    }

    pub fn find_segment_by_name<'a>(&'a mut self, name: &str) -> Option<&'a mut Segment> {
        self.segments.iter_mut().find(|seg| seg.get_name() == name)
    }
}

impl Segment {
    pub fn write_data(&mut self, data: &[u8]) -> Result<(), SegmentError> {
        if data.len() > self.info.size as usize {
            return Err(SegmentError::DataSizeTooLarge);
        }
        self.info.size = data.len() as u32;
        self.data_mgr
            .write_data(self.info.address, data)
            .map_err(SegmentError::WriteError)
    }

    pub fn read_data(&mut self) -> Result<Data, SegmentError> {
        let data = self
            .data_mgr
            .read_data(self.info.address, self.info.size as usize)
            .map_err(SegmentError::ReadError)?;

        if self.info.password_fingerprint.is_some() {
            Ok(Data::Binary(data))
        } else {
            self.info
                .data_type
                .from_bytes(&data)
                .map_err(SegmentError::DataError)
        }
    }

    pub fn update_meta(&mut self) -> Result<(), SegmentError> {
        let data = self.info.pack();
        self.data_mgr
            .write_data(self.meta_address, &data)
            .map_err(SegmentError::UpdateInfoError)
    }

    pub fn set_type(&mut self, data_type: DataType) -> Result<(), SegmentError> {
        self.info.data_type = data_type;
        self.update_meta()
    }

    pub fn rename(&mut self, new_name: &str) -> Result<(), SegmentError> {
        self.set_name(new_name);
        self.update_meta()
    }

    pub fn remove(&mut self) -> Result<(), SegmentError> {
        let address = self.info.address;
        let size = self.info.size as usize;

        self.data_mgr
            .write_zeroes(address, size)
            .map_err(SegmentError::WriteError)?;

        self.info = DataInfo::default();

        self.data_mgr
            .write_zeroes(self.meta_address, SEGMENT_INFO_SIZE)
            .map_err(SegmentError::UpdateInfoError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data_mgr::DataInterfaceType,
        seg_mgr::{EncryptionLevel, SegmentManager},
    };

    fn setup_mgr() -> SegmentManager {
        let mut mgr =
            SegmentManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.init_device(256, EncryptionLevel::Low).expect("device");
        mgr
    }

    #[test]
    fn segment_basic_operations() {
        let mut mgr = setup_mgr();
        mgr.set_segment("a", b"hi", DataType::Plain, None).unwrap();

        {
            let seg = mgr.find_segment_by_name("a").unwrap();
            assert_eq!(seg.read_data().unwrap(), Data::Plain("hi".to_string()));
            seg.rename("b").unwrap();
        }

        assert!(mgr.find_segment_by_name("a").is_none());
        {
            let seg = mgr.find_segment_by_name("b").unwrap();
            seg.set_type(DataType::Hex).unwrap();
            assert_eq!(seg.info.data_type, DataType::Hex);
            seg.remove().unwrap();
            assert!(!seg.is_active());
        }
    }

    #[test]
    fn remove_wipes_data_and_meta() {
        let mut mgr = setup_mgr();
        mgr.set_segment("a", b"secret", DataType::Plain, None)
            .unwrap();
        let mut data_mgr = mgr.data_mgr.clone();

        let (address, size, meta_address) = {
            let seg = mgr.find_segment_by_name("a").unwrap();
            let address = seg.info.address;
            let size = seg.info.size as usize;
            let meta_address = seg.meta_address;
            seg.remove().unwrap();
            (address, size, meta_address)
        };

        let wiped_data = data_mgr.read_data(address, size).unwrap();
        assert_eq!(wiped_data, vec![0u8; size]);

        let wiped_meta = data_mgr.read_data(meta_address, SEGMENT_INFO_SIZE).unwrap();
        assert_eq!(wiped_meta, vec![0u8; SEGMENT_INFO_SIZE]);
    }
}
