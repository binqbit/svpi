use std::sync::{Arc, RwLock};

use crate::{
    api::seg_mgmt::{DeviceStatus, RefDeviceStatus},
    seg_mgmt::DataType,
};
use rocket::{get, response::content::RawJson, State};
use serde_json::{json, Value};

#[get("/list")]
pub fn list(seg_mgmt: &State<Arc<RwLock<DeviceStatus>>>) -> RawJson<Value> {
    let mut seg_mgmt = seg_mgmt
        .inner()
        .write()
        .expect("Failed to lock segment manager");
    match seg_mgmt.as_mut_ref() {
        RefDeviceStatus::Some(seg_mgmt) => {
            let segments = seg_mgmt
                .get_segments_info()
                .into_iter()
                .map(|seg| {
                    json!({
                        "name": seg.get_name(),
                        "data_type": match seg.data_type {
                            DataType::Plain => "plain",
                            DataType::Encrypted => "encrypted",
                        },
                        "size": seg.size,
                    })
                })
                .collect::<Vec<Value>>();
            println!("[API::List] Segments: {}", segments.len());
            RawJson(json!({"status": "ok", "segments": segments}))
        }
        RefDeviceStatus::DeviceNotFound => {
            println!("[API::List] Device not found");
            RawJson(json!({"status": "device_not_found"}))
        }
        RefDeviceStatus::DeviceError => {
            println!("[API::List] Error loading segments");
            RawJson(json!({"status": "device_error"}))
        }
    }
}
