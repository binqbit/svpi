use serde::Serialize;

use crate::seg_mgr::{Data, DataType, Segment};

#[derive(Debug, Clone, Serialize)]
pub struct SegmentSummary {
    pub name: String,
    pub data_type: String,
    pub size: u32,
    pub fingerprint: String,
    pub password_fingerprint: Option<String>,
}

impl SegmentSummary {
    pub fn from_segment(seg: &Segment) -> Self {
        let password_fingerprint = seg.info.password_fingerprint.map(|fp| {
            Data::Binary(fp.to_vec())
                .to_string_typed(DataType::Hex)
                .unwrap_or_default()
        });
        Self {
            name: seg.get_name(),
            data_type: seg.info.data_type.to_string(),
            size: seg.info.size,
            fingerprint: seg.info.fingerprint.to_string(),
            password_fingerprint,
        }
    }
}
