use rocket::{get, response::content::RawJson};
use serde_json::{json, Value};

use crate::api::seg_mgmt::DeviceStatus;

#[get("/status")]
pub fn status() -> RawJson<Value> {
    let seg_mgmt = DeviceStatus::connect_device();
    match seg_mgmt {
        DeviceStatus::Some(mut seg_mgmt) => {
            let version = match seg_mgmt.get_version() {
                Ok(version) => version,
                Err(err) => {
                    println!("[API::Status] Error getting version: {}", err);
                    return RawJson(json!({"status": "device_error"}))
                },
            };
            println!("[API::Status] Device found with version: {}", version);
            RawJson(json!({
                "status": "ok",
                "version": version,
            }))
        },
        DeviceStatus::DeviceNotFound => {
            println!("[API::Status] Device not found");
            RawJson(json!({"status": "device_not_found"}))
        },
        DeviceStatus::DeviceError => {
            println!("[API::Status] Device error");
            RawJson(json!({"error": "device_error"}))
        },
    }
}
