use crate::{seg_mgmt::SegmentManager, spdm::SerialPortDataManager};

pub enum DeviceStatus {
    Some(SegmentManager),
    DeviceNotFound,
    DeviceError,
}

impl DeviceStatus {
    pub fn connect_device() -> Self {
        match SerialPortDataManager::find_device() {
            Ok(spdm) => {
                let mut seg_mgmt = spdm.into_segment_manager();
                match seg_mgmt.load_segments() {
                    Ok(_) => DeviceStatus::Some(seg_mgmt),
                    Err(_) => DeviceStatus::DeviceError,
                }
            },
            Err(_) => DeviceStatus::DeviceNotFound,
        }
    }
}