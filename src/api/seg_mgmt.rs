use std::{process::exit, sync::{Arc, RwLock}, thread, time::Duration};

use crate::{seg_mgmt::SegmentManager, spdm::SerialPortDataManager};

pub enum DeviceStatus {
    Some(SegmentManager),
    DeviceNotFound,
    DeviceError,
}

pub enum RefDeviceStatus<'a> {
    Some(&'a mut SegmentManager),
    DeviceNotFound,
    DeviceError,
}

impl DeviceStatus {
    pub fn connect_device() -> Self {
        match SerialPortDataManager::find_device() {
            Ok(spdm) => {
                let mut seg_mgmt: SegmentManager = spdm.into_segment_manager();
                match seg_mgmt.load_segments() {
                    Ok(_) => DeviceStatus::Some(seg_mgmt),
                    Err(_) => DeviceStatus::DeviceError,
                }
            },
            Err(_) => DeviceStatus::DeviceNotFound,
        }
    }

    pub fn as_mut_ref<'a>(&'a mut self) -> RefDeviceStatus {
        match self {
            DeviceStatus::Some(seg_mgmt) => RefDeviceStatus::Some(seg_mgmt),
            DeviceStatus::DeviceNotFound => RefDeviceStatus::DeviceNotFound,
            DeviceStatus::DeviceError => RefDeviceStatus::DeviceError,
        }
    }
}

pub fn start_connection_checking(seg_mgmt: Arc<RwLock<DeviceStatus>>) {
    let seg_mgmt = seg_mgmt.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let mut seg_mgmt = seg_mgmt.write().expect("Failed to lock segment manager");
            if let RefDeviceStatus::Some(seg_mgmt) = seg_mgmt.as_mut_ref() {
                if let Ok(res) = seg_mgmt.spdm.test(b"test") {
                    if res == b"test" {
                        continue;
                    }
                }
            }
            exit(1);
        }
    });
}