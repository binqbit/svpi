use rocket::{get, response::content::RawJson};
use serde_json::{json, Value};
use crate::{api::seg_mgmt::DeviceStatus, seg_mgmt::DataType};

#[get("/list")]
pub fn list() -> RawJson<Value> {
    let seg_mgmt = DeviceStatus::connect_device();
    match seg_mgmt {
        DeviceStatus::Some(seg_mgmt) => {
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
            RawJson(json!({"segments": segments}))
        },
        DeviceStatus::DeviceNotFound => {
            println!("[API::List] Device not found");
            RawJson(json!({"error": "device_not_found"}))
        },
        DeviceStatus::DeviceError => {
            println!("[API::List] Error loading segments");
            RawJson(json!({"error": "device_error"}))
        },
    }
}
