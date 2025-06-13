use std::sync::{Arc, RwLock};

use rocket::{get, response::content::RawJson, State};
use serde_json::{json, Value};

use crate::api::seg_mgmt::{DeviceStatus, RefDeviceStatus};

#[get("/status")]
pub fn status(seg_mgmt: &State<Arc<RwLock<DeviceStatus>>>) -> RawJson<Value> {
    let mut seg_mgmt = seg_mgmt
        .inner()
        .write()
        .expect("Failed to lock segment manager");
    match seg_mgmt.as_mut_ref() {
        RefDeviceStatus::Some(seg_mgmt) => {
            let version = match seg_mgmt.get_version() {
                Ok(version) => version,
                Err(err) => {
                    println!("[API::Status] Error getting version: {}", err);
                    return RawJson(json!({"status": "device_error"}));
                }
            };
            println!("[API::Status] Device found with version: {}", version);
            RawJson(json!({
                "status": "ok",
                "version": version,
            }))
        }
        RefDeviceStatus::DeviceNotFound => {
            println!("[API::Status] Device not found");
            RawJson(json!({"status": "device_not_found"}))
        }
        RefDeviceStatus::DeviceError => {
            println!("[API::Status] Device error");
            RawJson(json!({"status": "device_error"}))
        }
    }
}
