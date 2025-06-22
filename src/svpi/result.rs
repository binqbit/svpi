use crate::{
    seg_mgmt::{DataError, SegmentError},
    spdm::DeviceError,
};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    SegmentError(SegmentError),
    DeviceError(DeviceError),
    DataError(DataError),
    RootPasswordNotSet,
    PasswordIsRequired,
}

impl Into<Error> for SegmentError {
    fn into(self) -> Error {
        Error::SegmentError(self)
    }
}

impl Into<Error> for DeviceError {
    fn into(self) -> Error {
        Error::DeviceError(self)
    }
}

impl Into<Error> for DataError {
    fn into(self) -> Error {
        Error::DataError(self)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
