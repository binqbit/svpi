use serde::{Deserialize, Serialize};

use crate::seg_mgmt::SegmentManager;
use super::Status;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    status: Status,
    version: u32,
}

pub fn status(seg_mgmt: &mut SegmentManager) -> Result<StatusResponse, Status> {
    let version = match seg_mgmt.get_version() {
        Ok(version) => version,
        Err(_) => {
            return Err(Status::DeviceError);
        },
    };
    Ok(StatusResponse {
        status: Status::Ok,
        version,
    })
}
