use crate::{
    seg_mgmt::{Data, DataError, DataType},
    spdm::{DeviceError, RecordDirection},
    svpi::result::Error,
};

use super::{
    RawSegmentInfo, Segment, SegmentManager, ARCHITECTURE_VERSION, END_INIT_DATA, START_INIT_DATA,
};

impl SegmentManager {
    pub fn check_init_data(&mut self) -> Result<bool, DeviceError> {
        let start = self
            .spdm
            .read_data(self.start_init_data_address(), START_INIT_DATA.len() as u32)?;
        if start != START_INIT_DATA {
            return Ok(false);
        }

        let end = self
            .spdm
            .read_data(self.end_init_data_address(), END_INIT_DATA.len() as u32)?;
        if end != END_INIT_DATA {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn check_architecture_version(&mut self) -> Result<bool, DeviceError> {
        let version = self.get_version()?;
        Ok(version == ARCHITECTURE_VERSION)
    }

    pub fn get_version(&mut self) -> Result<u32, DeviceError> {
        self.spdm.read_value(self.version_address())
    }

    pub fn set_version(&mut self, version: u32) -> Result<(), DeviceError> {
        self.spdm.write_value(self.version_address(), version)
    }

    pub fn get_memory_size(&mut self) -> Result<u32, DeviceError> {
        self.spdm.read_value(self.memory_size_address())
    }

    pub fn set_memory_size(&mut self, memory_size: u32) -> Result<(), DeviceError> {
        self.spdm
            .write_value(self.memory_size_address(), memory_size)
    }

    pub fn get_root_password(
        &mut self,
        decrypt_password: &str,
        encrypt_password: Option<&str>,
    ) -> Result<String, Error> {
        let data = self
            .spdm
            .read_values(self.root_password_address(), RecordDirection::Right)
            .map_err(Error::DeviceError)?;

        if data.is_empty() {
            return Err(Error::RootPasswordNotSet);
        }

        let decrypted = DataType::Plain
            .from_bytes(&data, true, Some(decrypt_password))
            .map_err(Error::DataError)?;

        let decrypted = match decrypted {
            Data::Encrypted(_) => {
                return Err(Error::DataError(DataError::DecryptionError));
            }
            Data::Plain(p) => p,
            _ => panic!("Expected plain data type for root password"),
        };

        if let Some(encrypt_password) = encrypt_password {
            let encrypted = Data::Plain(decrypted)
                .to_bytes(Some(encrypt_password))
                .map_err(Error::DataError)?;

            let encrypted_b58 = DataType::Base58
                .from_bytes(&encrypted, false, None)
                .map_err(Error::DataError)?;

            let encrypted_b58 = match encrypted_b58 {
                Data::Base58(s) => s,
                _ => panic!("Expected base58 data type for root password"),
            };

            Ok(encrypted_b58)
        } else {
            Ok(decrypted)
        }
    }

    pub fn set_root_password(
        &mut self,
        password: &str,
        encrypt_password: &str,
    ) -> Result<(), Error> {
        let password = Data::Plain(password.to_string())
            .to_bytes(Some(encrypt_password))
            .map_err(Error::DataError)?;

        self.spdm
            .write_values(
                self.root_password_address(),
                &password,
                RecordDirection::Right,
            )
            .map_err(Error::DeviceError)
    }

    pub fn reset_root_password(&mut self) -> Result<(), DeviceError> {
        self.spdm
            .write_values::<u8>(self.root_password_address(), &[], RecordDirection::Right)
    }

    pub fn is_root_password_set(&mut self) -> Result<bool, DeviceError> {
        self.spdm
            .read_values::<u8>(self.root_password_address(), RecordDirection::Right)
            .map(|data| !data.is_empty())
    }

    pub fn save_segments_count(&mut self) -> Result<(), DeviceError> {
        self.spdm
            .write_value(self.segments_info_address(), self.segments.len() as u32)
    }

    pub fn save_segment_meta(&mut self, seg: &Segment) -> Result<(), DeviceError> {
        self.spdm.write_value(seg.meta_address, seg.to_raw())
    }

    pub fn add_segment_meta(&mut self, segment: Segment) -> Result<(), DeviceError> {
        self.save_segment_meta(&segment)?;
        self.segments.insert(0, segment);
        self.save_segments_count()?;
        Ok(())
    }

    pub fn init_segments(&mut self, memory_size: u32) -> Result<(), DeviceError> {
        self.memory_size = memory_size;
        self.spdm
            .write_data(self.start_init_data_address(), START_INIT_DATA)?;
        self.spdm
            .write_data(self.end_init_data_address(), END_INIT_DATA)?;
        self.set_version(self.version)?;
        self.set_memory_size(memory_size)?;
        self.reset_root_password()?;
        let raw_segments: Vec<RawSegmentInfo> = self.segments.iter().map(Segment::to_raw).collect();
        self.spdm.write_values(
            self.segments_info_address(),
            &raw_segments,
            RecordDirection::Left,
        )
    }

    pub fn load_segments(&mut self) -> Result<(), DeviceError> {
        self.version = self.get_version()?;
        self.memory_size = self.get_memory_size()?;
        let raw_segments = self
            .spdm
            .read_values::<RawSegmentInfo>(self.segments_info_address(), RecordDirection::Left)?;
        self.segments.clear();
        for (i, segment) in raw_segments.into_iter().rev().enumerate() {
            let meta_address = self.segment_meta_address(i as u32);
            let segment = Segment::from_raw(self.spdm.clone(), meta_address, segment);
            self.segments.insert(0, segment);
        }
        Ok(())
    }

    pub fn format_data(&mut self) -> Result<(), DeviceError> {
        let data = vec![0u8; self.memory_size as usize];
        self.spdm.write_data(0, &data)
    }

    pub fn get_active_segments<'a>(&'a self) -> Vec<&'a Segment> {
        self.segments
            .iter()
            .filter(|segment| segment.is_active)
            .collect()
    }

    pub fn get_active_segments_mut<'a>(&'a mut self) -> Vec<&'a mut Segment> {
        self.segments
            .iter_mut()
            .filter(|segment| segment.is_active)
            .collect::<Vec<&mut Segment>>()
    }

    pub fn get_dump(&mut self) -> Result<Vec<u8>, DeviceError> {
        let start_address = self.start_init_data_address();
        self.spdm.read_data(start_address, self.memory_size as u32)
    }

    pub fn set_dump(&mut self, data: &[u8]) -> Result<(), DeviceError> {
        let start_address = self.start_init_data_address();
        self.spdm.write_data(start_address, data)
    }
}
