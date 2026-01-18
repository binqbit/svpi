use serialport_srwp::SerialPortDataManager;
use thiserror::Error;

use crate::data_mgr::{
    fs::FileSystemDataManager, memory::MemoryDataManager, serial_port::SerialPortExt,
};

mod fs;
mod io;
mod memory;
mod serial_port;

pub use io::*;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Failed to read from device")]
    ReadError,
    #[error("Failed to write to device")]
    WriteError,
    #[error("Failed to initialize memory")]
    InitMemoryError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataInterfaceType {
    SerialPort,
    FileSystem(String),
    Memory(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum DataManager {
    SerialPort(SerialPortDataManager),
    FileSystem(FileSystemDataManager),
    Memory(MemoryDataManager),
}

impl DataInterfaceType {
    pub fn load_data_manager(&self) -> Result<DataManager, DeviceError> {
        match self {
            DataInterfaceType::SerialPort => {
                SerialPortDataManager::find_device().map(DataManager::SerialPort)
            }
            DataInterfaceType::FileSystem(path) => {
                FileSystemDataManager::open_file(&path).map(DataManager::FileSystem)
            }
            DataInterfaceType::Memory(data) => {
                Ok(DataManager::Memory(MemoryDataManager::new(data.clone())))
            }
        }
    }
}

impl Default for DataInterfaceType {
    fn default() -> Self {
        DataInterfaceType::SerialPort
    }
}

impl DataManager {
    pub fn init_memory(&mut self, size: usize) -> Result<(), DeviceError> {
        match self {
            DataManager::SerialPort(_) => Ok(()),
            DataManager::FileSystem(mgr) => mgr.init_memory(size),
            DataManager::Memory(mgr) => mgr.init_memory(size),
        }
    }
}

impl DataManagerExt for DataManager {
    fn read_data(&mut self, address: u32, size: usize) -> Result<Vec<u8>, DeviceError> {
        match self {
            DataManager::SerialPort(manager) => manager.read_data(address, size),
            DataManager::FileSystem(manager) => manager.read_data(address, size),
            DataManager::Memory(manager) => manager.read_data(address, size),
        }
    }

    fn write_data(&mut self, address: u32, data: &[u8]) -> Result<(), DeviceError> {
        match self {
            DataManager::SerialPort(manager) => manager.write_data(address, data),
            DataManager::FileSystem(manager) => manager.write_data(address, data),
            DataManager::Memory(manager) => manager.write_data(address, data),
        }
    }
}
