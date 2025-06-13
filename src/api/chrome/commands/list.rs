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

#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentResponse {
    name: String,
    data_type: DataTypeResponse,
    size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    status: Status,
    segments: Vec<SegmentResponse>,
}

pub fn list(seg_mgmt: &mut SegmentManager) -> Result<ListResponse, Status> {
    let segments = seg_mgmt
        .get_segments_info()
        .into_iter()
        .map(|seg| SegmentResponse {
            name: seg.get_name(),
            data_type: match seg.data_type {
                DataType::Plain => DataTypeResponse::Plain,
                DataType::Encrypted => DataTypeResponse::Encrypted,
            },
            size: seg.size,
        })
        .collect::<Vec<SegmentResponse>>();
    Ok(ListResponse {
        status: Status::Ok,
        segments,
    })
}
