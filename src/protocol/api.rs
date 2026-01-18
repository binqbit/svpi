use serde::{Deserialize, Serialize};
use serde_json::json;

use super::segments::SegmentSummary;

use crate::{
    data_mgr::DataInterfaceType,
    pass_mgr::PasswordManager,
    seg_mgr::{DataType, ARCHITECTURE_VERSION},
    utils::response::SvpiResponse,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiTransport {
    Server,
    Chrome,
}

impl ApiTransport {
    pub const fn namespace(self) -> &'static str {
        match self {
            ApiTransport::Server => "api",
            ApiTransport::Chrome => "chrome",
        }
    }
}

fn cmd(transport: ApiTransport, name: &'static str) -> Option<String> {
    Some(format!("{}.{}", transport.namespace(), name))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandRequest {
    #[serde(rename = "status")]
    Status {},
    #[serde(rename = "list")]
    List {},
    #[serde(rename = "get_data")]
    GetData(GetDataRequest),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDataRequest {
    pub name: String,
    pub password: Option<String>,
}

pub fn status(transport: ApiTransport, interface_type: DataInterfaceType) -> SvpiResponse {
    let command = cmd(transport, "status");

    let mut pass_mgr = match PasswordManager::from_device_type(interface_type) {
        Ok(mgr) => mgr,
        Err(err) => return SvpiResponse::data_manager_error_public(command, err),
    };

    let seg_mgr = pass_mgr.get_data_manager();
    let found_version = seg_mgr.read_architecture_version().unwrap_or(0);

    let initialized = match seg_mgr.check_init_data() {
        Ok(v) => v,
        Err(err) => return SvpiResponse::err(command, "device_error", err.to_string(), None),
    };
    if !initialized {
        return SvpiResponse::err(
            command,
            "device_not_initialized",
            "Device not initialized",
            Some(json!({ "architecture_version": found_version })),
        );
    }

    let architecture_ok = match seg_mgr.check_architecture_version() {
        Ok(v) => v,
        Err(err) => return SvpiResponse::err(command, "device_error", err.to_string(), None),
    };
    if !architecture_ok {
        return SvpiResponse::err(
            command,
            "architecture_mismatch",
            "Architecture mismatch",
            Some(json!({ "expected": ARCHITECTURE_VERSION, "found": found_version })),
        );
    }

    SvpiResponse::ok(
        command,
        json!({ "status": "ok", "architecture_version": found_version }),
    )
}

pub fn list(transport: ApiTransport, interface_type: DataInterfaceType) -> SvpiResponse {
    let command = cmd(transport, "list");

    let mut pass_mgr = match PasswordManager::try_load(interface_type) {
        Ok(mgr) => mgr,
        Err(err) => return SvpiResponse::data_manager_error_public(command, err),
    };

    let segments = pass_mgr
        .get_data_manager()
        .get_active_segments()
        .into_iter()
        .filter(|seg| seg.info.data_type != DataType::EncryptionKey)
        .map(|seg| SegmentSummary::from_segment(&seg))
        .collect::<Vec<_>>();

    SvpiResponse::ok(command, json!({ "segments": segments }))
}

pub fn get_data(
    transport: ApiTransport,
    interface_type: DataInterfaceType,
    req: GetDataRequest,
) -> SvpiResponse {
    let command = cmd(transport, "get");

    if req.name.trim().is_empty() {
        return SvpiResponse::missing_argument(command, "name");
    }

    let mut pass_mgr = match PasswordManager::try_load(interface_type) {
        Ok(mgr) => mgr,
        Err(err) => return SvpiResponse::data_manager_error_public(command, err),
    };

    let name = req.name;
    let (encrypted, data_type) = {
        let seg = pass_mgr.get_data_manager().find_segment_by_name(&name);
        let Some(seg) = seg else {
            return SvpiResponse::err(
                command,
                "data_not_found",
                format!("Data '{name}' not found"),
                None,
            );
        };
        (seg.info.password_fingerprint.is_some(), seg.info.data_type)
    };

    if data_type == DataType::EncryptionKey {
        return SvpiResponse::err(
            command,
            "forbidden",
            "Encryption keys are not readable via API".to_string(),
            None,
        );
    }

    if encrypted && req.password.as_deref().unwrap_or("").is_empty() {
        return SvpiResponse::err(
            command,
            "password_required",
            "Password required for decryption".to_string(),
            Some(json!({ "name": name })),
        );
    }

    let password = req.password.filter(|p| !p.is_empty());
    match pass_mgr.read_password(&name, || password.clone().unwrap_or_default()) {
        Ok(data) => SvpiResponse::ok(
            command,
            json!({
                "name": name,
                "data": data,
                "data_type": data_type.to_string(),
                "encrypted": encrypted,
            }),
        ),
        Err(err) => SvpiResponse::password_manager_error(command, err),
    }
}

pub fn handle(
    transport: ApiTransport,
    interface_type: DataInterfaceType,
    request: CommandRequest,
) -> SvpiResponse {
    match request {
        CommandRequest::Status {} => status(transport, interface_type),
        CommandRequest::List {} => list(transport, interface_type),
        CommandRequest::GetData(req) => get_data(transport, interface_type, req),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data_mgr::DataInterfaceType, pass_mgr::PasswordManager, seg_mgr::EncryptionLevel};

    fn seeded_dump() -> Vec<u8> {
        let mut pass_mgr =
            PasswordManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init mgr");
        pass_mgr
            .get_data_manager()
            .init_device(1024)
            .expect("init device");

        // Plain value
        pass_mgr
            .save_password("plain", "alpha", None)
            .expect("save plain");

        // Encrypted value
        pass_mgr
            .save_password("secret", "bravo", Some("pw".to_string()))
            .expect("save encrypted");

        // Encryption key segment (must be hidden/forbidden for API)
        pass_mgr.set_master_password("master").expect("set master");
        pass_mgr
            .add_encryption_key("master", "key", "keypw", EncryptionLevel::default())
            .expect("add key");

        pass_mgr.get_data_manager().get_dump().expect("dump")
    }

    #[test]
    fn status_ok_on_initialized_device() {
        let dump = seeded_dump();
        let resp = status(ApiTransport::Server, DataInterfaceType::Memory(dump));
        assert!(resp.ok);
        let result = resp.result.expect("result");
        assert_eq!(result["status"], "ok");
    }

    #[test]
    fn status_reports_not_initialized() {
        let resp = status(
            ApiTransport::Server,
            DataInterfaceType::Memory(vec![0u8; 256]),
        );
        assert!(!resp.ok);
        assert_eq!(resp.error.as_ref().unwrap().code, "device_not_initialized");
    }

    #[test]
    fn list_filters_encryption_key_segments() {
        let dump = seeded_dump();
        let resp = list(ApiTransport::Server, DataInterfaceType::Memory(dump));
        assert!(resp.ok);
        let result = resp.result.expect("result");
        let segments = result["segments"].as_array().expect("segments");

        let names = segments
            .iter()
            .filter_map(|item| item["name"].as_str())
            .collect::<Vec<_>>();

        assert!(names.contains(&"plain"));
        assert!(names.contains(&"secret"));
        assert!(!names.contains(&"key"));
    }

    #[test]
    fn get_data_forbidden_for_encryption_key() {
        let dump = seeded_dump();
        let resp = get_data(
            ApiTransport::Server,
            DataInterfaceType::Memory(dump),
            GetDataRequest {
                name: "key".to_string(),
                password: None,
            },
        );
        assert!(!resp.ok);
        assert_eq!(resp.error.as_ref().unwrap().code, "forbidden");
    }

    #[test]
    fn get_data_requires_password_for_encrypted_segments() {
        let dump = seeded_dump();
        let resp = get_data(
            ApiTransport::Server,
            DataInterfaceType::Memory(dump),
            GetDataRequest {
                name: "secret".to_string(),
                password: None,
            },
        );
        assert!(!resp.ok);
        assert_eq!(resp.error.as_ref().unwrap().code, "password_required");
    }

    #[test]
    fn get_data_roundtrip_plain_and_encrypted() {
        let dump = seeded_dump();

        let plain = get_data(
            ApiTransport::Server,
            DataInterfaceType::Memory(dump.clone()),
            GetDataRequest {
                name: "plain".to_string(),
                password: None,
            },
        );
        assert!(plain.ok);
        assert_eq!(plain.result.as_ref().unwrap()["data"], "alpha");

        let encrypted_ok = get_data(
            ApiTransport::Server,
            DataInterfaceType::Memory(dump),
            GetDataRequest {
                name: "secret".to_string(),
                password: Some("pw".to_string()),
            },
        );
        assert!(encrypted_ok.ok);
        assert_eq!(encrypted_ok.result.as_ref().unwrap()["data"], "bravo");
    }
}
