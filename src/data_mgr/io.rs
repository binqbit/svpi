use crate::data_mgr::DeviceError;

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum RecordDirection {
//     Right,
//     Left,
// }

pub trait DataManagerExt {
    fn read_data(&mut self, address: u32, size: usize) -> Result<Vec<u8>, DeviceError>;
    fn write_data(&mut self, address: u32, data: &[u8]) -> Result<(), DeviceError>;

    fn write_zeroes(&mut self, address: u32, len: usize) -> Result<(), DeviceError> {
        if len == 0 {
            return Ok(());
        }

        let buffer = vec![0u8; len];
        self.write_data(address, &buffer)
    }

    fn read_value<T: Sized>(&mut self, address: u32) -> Result<T, DeviceError> {
        let size = std::mem::size_of::<T>();
        let data = self.read_data(address, size)?;
        if data.len() < size {
            return Err(DeviceError::ReadError);
        }
        let value = unsafe { std::ptr::read_unaligned(data.as_ptr() as *const T) };
        Ok(value)
    }

    // fn read_values<T: Sized>(
    //     &mut self,
    //     address: u32,
    //     record_direction: RecordDirection,
    // ) -> Result<Vec<T>, DeviceError> {
    //     let size = self.read_value::<u32>(address)? as usize;
    //     if size == 0 {
    //         return Ok(Vec::new());
    //     }
    //     let value_size = std::mem::size_of::<T>();
    //     let data = match record_direction {
    //         RecordDirection::Right => self.read_data(address + 4, size * value_size)?,
    //         RecordDirection::Left => {
    //             self.read_data(address - (size * value_size) as u32, size * value_size)?
    //         }
    //     };
    //     if data.len() < size * value_size {
    //         return Err(DeviceError::ReadError);
    //     }
    //     let mut result = Vec::new();
    //     for i in 0..size {
    //         let value =
    //             unsafe { std::ptr::read_unaligned(data[i * value_size..].as_ptr() as *const T) };
    //         result.push(value);
    //     }
    //     Ok(result)
    // }

    fn write_value<T: Sized>(&mut self, address: u32, value: T) -> Result<(), DeviceError> {
        let size = std::mem::size_of::<T>();
        let data = unsafe { std::slice::from_raw_parts(&value as *const T as *const u8, size) };
        self.write_data(address, data)
    }

    // fn write_values<T: Sized>(
    //     &mut self,
    //     address: u32,
    //     values: &[T],
    //     record_direction: RecordDirection,
    // ) -> Result<(), DeviceError> {
    //     let len: u32 = values, Eq)]
    // pub enum RecordDirection {
    //     Right,
    //     Left,
    // }
    //         .len()
    //         .try_into()
    //         .map_err(|_| DeviceError::WriteError)?;
    //     self.write_value(address, len)?;
    //     if values.is_empty() {
    //         return Ok(());
    //     }
    //     let value_size = std::mem::size_of::<T>();
    //     let mut data = Vec::new();
    //     for item in values {
    //         let bytes =
    //             unsafe { std::slice::from_raw_parts(item as *const T as *const u8, value_size) };
    //         data.extend_from_slice(bytes);
    //     }
    //     match record_direction {
    //         RecordDirection::Right => self.write_data(address + 4, &data),
    //         RecordDirection::Left => {
    //             self.write_data(address - std::mem::size_of_val(values) as u32, &data)
    //         }
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_mgr::{memory::MemoryDataManager, DataManager};

    #[test]
    fn write_zeroes_writes_expected_bytes() {
        let mut mgr = DataManager::Memory(MemoryDataManager::new(vec![0xAA; 64]));

        mgr.write_zeroes(10, 20).unwrap();

        let wiped = mgr.read_data(10, 20).unwrap();
        assert_eq!(wiped, vec![0u8; 20]);

        let prefix = mgr.read_data(0, 10).unwrap();
        assert_eq!(prefix, vec![0xAA; 10]);

        let suffix = mgr.read_data(30, 34).unwrap();
        assert_eq!(suffix, vec![0xAA; 34]);
    }

    #[test]
    fn write_value_read_value_roundtrip_u32() {
        let mut mgr = DataManager::Memory(MemoryDataManager::new(vec![0u8; 64]));

        mgr.write_value(8, 0xDEADBEEFu32).unwrap();
        let v: u32 = mgr.read_value(8).unwrap();
        assert_eq!(v, 0xDEADBEEFu32);
    }

    #[test]
    fn read_value_out_of_bounds_returns_error() {
        let mut mgr = DataManager::Memory(MemoryDataManager::new(vec![0u8; 16]));
        assert!(mgr.read_value::<u32>(1000).is_err());
    }
}
