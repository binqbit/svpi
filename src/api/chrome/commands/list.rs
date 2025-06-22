use serde::{Deserialize, Serialize};

use super::Status;
use crate::seg_mgmt::{DataType, SegmentManager};

#[derive(Debug, Serialize, Deserialize)]
pub enum DataTypeResponse {
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "encrypted")]
    Encrypted,
}

#[derive(Debug, Serialize)]
pub struct SegmentResponse {
    name: String,
    data_type: DataType,
    size: u32,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    status: Status,
    segments: Vec<SegmentResponse>,
}

pub fn list(seg_mgmt: &mut SegmentManager) -> Result<ListResponse, Status> {
    let segments = seg_mgmt
        .get_active_segments()
        .into_iter()
        .map(|seg| SegmentResponse {
            name: seg.get_name(),
            data_type: seg.data_type,
            size: seg.size,
        })
        .collect::<Vec<SegmentResponse>>();
    Ok(ListResponse {
        status: Status::Ok,
        segments,
    })
}
