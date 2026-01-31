use serde_json::{json, Value};

use crate::{
    pass_mgr::PasswordManagerError,
    seg_mgr::{DataError, DataManagerError, SegmentError},
    utils::response::{SvpiError, SvpiResponse},
};

impl SvpiError {
    pub fn exit_code(&self) -> i32 {
        match self.code.as_str() {
            "invalid_command"
            | "missing_argument"
            | "invalid_argument"
            | "confirmation_required" => 2,
            "device_not_found" => 3,
            "device_not_initialized" => 4,
            "architecture_mismatch" => 5,
            "data_not_found" => 6,
            "password_required" | "password_error" | "master_password_invalid" => 7,
            "not_enough_memory" => 8,
            _ => 1,
        }
    }
}

impl SvpiResponse {
    pub fn exit_code(&self) -> i32 {
        if self.ok {
            0
        } else {
            self.error.as_ref().map(|err| err.exit_code()).unwrap_or(1)
        }
    }

    pub fn with_exit_code(self) -> (Self, i32) {
        let code = self.exit_code();
        (self, code)
    }

    pub fn data_manager_error_public(cmd: Option<String>, err: DataManagerError) -> Self {
        match err {
            DataManagerError::DeviceNotFound(_) => {
                SvpiResponse::err(cmd, "device_not_found", "Device not found", None)
            }
            DataManagerError::InvalidArgument(message) => {
                SvpiResponse::invalid_argument(cmd, "device", message)
            }
            DataManagerError::DeviceNotInitialized => SvpiResponse::err(
                cmd,
                "device_not_initialized",
                "Device not initialized",
                None,
            ),
            DataManagerError::MismatchArchitectureVersion => {
                SvpiResponse::err(cmd, "architecture_mismatch", "Architecture mismatch", None)
            }
            DataManagerError::DeviceError(_) => {
                SvpiResponse::err(cmd, "device_error", "Device error", None)
            }
        }
    }

    pub fn data_manager_error_verbose(cmd: Option<String>, err: DataManagerError) -> Self {
        let (code, message) = match &err {
            DataManagerError::DeviceNotFound(_) => ("device_not_found", err.to_string()),
            DataManagerError::InvalidArgument(_) => ("invalid_argument", err.to_string()),
            DataManagerError::DeviceNotInitialized => ("device_not_initialized", err.to_string()),
            DataManagerError::MismatchArchitectureVersion => {
                ("architecture_mismatch", err.to_string())
            }
            DataManagerError::DeviceError(_) => ("device_error", err.to_string()),
        };
        SvpiResponse::err(cmd, code, message, None)
    }

    pub fn password_manager_error(cmd: Option<String>, err: PasswordManagerError) -> Self {
        let (code, message) = match &err {
            PasswordManagerError::ReadPasswordError(SegmentError::NotFound(_)) => {
                ("data_not_found", err.to_string())
            }
            PasswordManagerError::InvalidEncryptionKey(_) => ("password_error", err.to_string()),
            PasswordManagerError::EncryptionError(_) => ("password_error", err.to_string()),
            PasswordManagerError::GetEncryptionKey(_) => ("password_error", err.to_string()),
            PasswordManagerError::ReadPasswordError(SegmentError::DataError(
                DataError::DecryptionError,
            )) => ("password_error", err.to_string()),
            _ => ("device_error", err.to_string()),
        };
        SvpiResponse::err(cmd, code, message, None)
    }

    pub fn missing_argument(cmd: Option<String>, name: &str) -> Self {
        SvpiResponse::err(
            cmd,
            "missing_argument",
            format!("Missing argument: {name}"),
            Some(json!({ "name": name })),
        )
    }

    pub fn invalid_argument(cmd: Option<String>, name: &str, message: impl Into<String>) -> Self {
        SvpiResponse::err(
            cmd,
            "invalid_argument",
            message,
            Some(json!({ "name": name })),
        )
    }

    pub fn confirmation_required(cmd: Option<String>, action: &str, details: Value) -> Self {
        SvpiResponse::err(
            cmd,
            "confirmation_required",
            "Confirmation required (pass --confirm)".to_string(),
            Some(json!({ "action": action, "context": details })),
        )
    }
}
