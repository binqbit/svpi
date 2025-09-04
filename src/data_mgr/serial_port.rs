use serialport_srwp::{AddressedIo, SerialPortDataManager};

use crate::data_mgr::DeviceError;

pub trait SerialPortExt {
    fn find_device() -> Result<Self, DeviceError>
    where
        Self: Sized;

    fn read_data(&mut self, offset: u32, size: usize) -> Result<Vec<u8>, DeviceError>;

    fn write_data(&mut self, address: u32, data: &[u8]) -> Result<(), DeviceError>;
}

impl SerialPortExt for SerialPortDataManager {
    fn find_device() -> Result<Self, DeviceError> {
        let devices =
            SerialPortDataManager::find_devices().map_err(|_| DeviceError::DeviceNotFound)?;

        for device in devices {
            if let Ok(mut port) = device.connect() {
                if let Ok(result) = port.test(b"hello_world") {
                    if result.as_slice() == b"hello_world" {
                        return Ok(port);
                    }
                }
            }
        }

        Err(DeviceError::DeviceNotFound)
    }

    fn read_data(&mut self, offset: u32, size: usize) -> Result<Vec<u8>, DeviceError> {
        AddressedIo::read_data(self, offset, size).map_err(|_| DeviceError::ReadError)
    }

    fn write_data(&mut self, address: u32, data: &[u8]) -> Result<(), DeviceError> {
        AddressedIo::write_data(self, address, data).map_err(|_| DeviceError::WriteError)
    }
}
