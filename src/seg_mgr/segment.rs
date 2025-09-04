use thiserror::Error;

use crate::{
    data_mgr::{DataManagerExt, DeviceError},
    seg_mgr::{
        Data, DataError, DataInfo, DataManager, DataType, SegmentManager, DATA_FINGERPRINT_SIZE,
        DATA_NAME_SIZE,
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

    pub fn disable(&mut self) {
        self.info.name = [0; DATA_NAME_SIZE];
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
        self.data_mgr
            .write_value(self.meta_address, self.info)
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
        let zero_data = vec![0u8; self.info.size as usize];
        self.disable();
        self.write_data(&zero_data)?;
        self.update_meta()
    }
}
