use crate::{
    seg_mgmt::DataError,
    spdm::{DeviceError, SerialPortDataManager},
};

use super::{Data, DataType, RawSegmentInfo, Segment, SegmentManager};

#[derive(Debug)]
pub enum SegmentError {
    DataSizeTooLarge,
    NotFound,
    DataError(DataError),
    UpdateInfoError(DeviceError),
    WriteError(DeviceError),
    ReadError(DeviceError),
}

impl Segment {
    pub fn new(
        spdm: SerialPortDataManager,
        meta_address: u32,
        address: u32,
        size: u32,
        data_type: DataType,
        is_encrypted: bool,
        name: &str,
    ) -> Segment {
        let name = name.as_bytes();
        let mut name_buf = [0; 32];
        name_buf[..name.len()].copy_from_slice(name);
        Segment {
            spdm,
            meta_address,
            address,
            size,
            data_type,
            is_encrypted,
            is_active: true,
            name: name_buf,
        }
    }

    pub fn from_raw(
        spdm: SerialPortDataManager,
        meta_address: u32,
        raw: RawSegmentInfo,
    ) -> Segment {
        Segment {
            spdm,
            meta_address,
            address: raw.0,
            size: raw.1,
            data_type: raw.2,
            is_encrypted: raw.3,
            is_active: raw.4,
            name: raw.5,
        }
    }

    pub fn to_raw(&self) -> RawSegmentInfo {
        (
            self.address,
            self.size,
            self.data_type,
            self.is_encrypted,
            self.is_active,
            self.name,
        )
    }

    pub fn set_address(&mut self, address: u32) {
        self.address = address;
    }

    pub fn disable(&mut self) {
        self.is_active = false;
    }

    pub fn set_encrypted(&mut self, is_encrypted: bool) {
        self.is_encrypted = is_encrypted;
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
    pub fn set_segment<'a>(
        &'a mut self,
        name: &str,
        data: Data,
        password: Option<&str>,
    ) -> Result<Option<&'a Segment>, SegmentError> {
        let data_type = data.get_type();
        let data = data.to_bytes(password).map_err(SegmentError::DataError)?;
        let address = self.find_new_segment_address(data.len() as u32);
        let meta_address = self.segment_meta_address(self.segments.len() as u32);

        if let Some(old_seg) = self.find_segment_by_name(name) {
            old_seg.remove()?;
        }

        if let Some(address) = address {
            let mut segment = Segment::new(
                self.spdm.clone(),
                meta_address,
                address,
                data.len() as u32,
                data_type,
                password.is_some(),
                name,
            );
            segment.write_data(&data)?;
            self.add_segment_meta(segment)
                .map_err(SegmentError::UpdateInfoError)?;
            Ok(self.segments.first())
        } else {
            Ok(None)
        }
    }

    pub fn find_segment_by_name<'a>(&'a mut self, name: &str) -> Option<&'a mut Segment> {
        self.segments
            .iter_mut()
            .find(|seg| seg.is_active && seg.get_name() == name)
    }
}

impl Segment {
    pub fn write_data(&mut self, data: &[u8]) -> Result<(), SegmentError> {
        if data.len() as u32 > self.size {
            return Err(SegmentError::DataSizeTooLarge);
        }
        self.size = data.len() as u32;
        self.spdm
            .write_data(self.address, data)
            .map_err(SegmentError::WriteError)
    }

    pub fn read_data(&mut self, password: Option<&str>) -> Result<Data, SegmentError> {
        let data = self
            .spdm
            .read_data(self.address, self.size)
            .map_err(SegmentError::ReadError)?;

        self.data_type
            .from_bytes(&data, self.is_encrypted, password)
            .map_err(SegmentError::DataError)
    }

    pub fn set_type(&mut self, data_type: DataType) -> Result<(), SegmentError> {
        self.data_type = data_type;
        self.update_meta()
    }

    pub fn remove(&mut self) -> Result<(), SegmentError> {
        self.disable();
        self.update_meta()
    }

    pub fn update_meta(&mut self) -> Result<(), SegmentError> {
        self.spdm
            .write_value(self.meta_address, &self.to_raw())
            .map_err(SegmentError::UpdateInfoError)
    }
}
