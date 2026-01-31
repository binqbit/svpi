use serde::Serialize;
use serde_json::{json, Value};

use crate::seg_mgr::ARCHITECTURE_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Cli,
    Json,
}

pub const RESPONSE_SCHEMA_V1: &str = "svpi.response.v1";

fn dump_protection_label(code: u64) -> &'static str {
    match code {
        0 => "none",
        1 => "low",
        2 => "medium",
        3 => "strong",
        4 => "hardened",
        _ => "unknown",
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SvpiMeta {
    pub app_version: &'static str,
    pub architecture_version: u32,
}

impl Default for SvpiMeta {
    fn default() -> Self {
        Self {
            app_version: env!("CARGO_PKG_VERSION"),
            architecture_version: ARCHITECTURE_VERSION,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SvpiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SvpiResponse {
    pub schema: &'static str,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SvpiError>,
    pub meta: SvpiMeta,
}

impl SvpiResponse {
    pub fn ok(command: Option<String>, result: Value) -> Self {
        Self {
            schema: RESPONSE_SCHEMA_V1,
            ok: true,
            command,
            result: Some(result),
            error: None,
            meta: SvpiMeta::default(),
        }
    }

    pub fn cancelled(command: Option<String>, action: &str, context: Value) -> Self {
        SvpiResponse::ok(
            command,
            json!({ "cancelled": true, "action": action, "context": context }),
        )
    }

    pub fn err(
        command: Option<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<Value>,
    ) -> Self {
        Self {
            schema: RESPONSE_SCHEMA_V1,
            ok: false,
            command,
            result: None,
            error: Some(SvpiError {
                code: code.into(),
                message: message.into(),
                details,
            }),
            meta: SvpiMeta::default(),
        }
    }

    pub fn print(&self, format: OutputFormat) {
        match format {
            OutputFormat::Json => self.print_json(),
            OutputFormat::Cli => self.print_cli(),
        }
    }

    pub fn print_json(&self) {
        println!("{}", self.to_json_string());
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|err| {
            let fallback = SvpiResponse::err(
                self.command.clone(),
                "internal_error",
                format!("Failed to serialize JSON response: {err}"),
                Some(json!({ "command": self.command })),
            );
            serde_json::to_string(&fallback).unwrap_or_else(|_| "{}".to_string())
        })
    }

    pub fn print_cli(&self) {
        if !self.ok {
            let (code, message, details) = self
                .error
                .as_ref()
                .map(|e| (e.code.as_str(), e.message.as_str(), e.details.as_ref()))
                .unwrap_or(("error", "Unknown error", None));

            eprintln!("{code}: {message}");
            if let Some(details) = details {
                Self::print_error_details_cli(details);
            }
            return;
        }

        let Some(result) = self.result.as_ref() else {
            println!("OK");
            return;
        };

        if result
            .get("cancelled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            println!("Cancelled.");
            return;
        }

        let cmd = self
            .command
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();

        match cmd.as_str() {
            "help" => {
                self.print_info_cli();
                println!();

                let commands = result
                    .get("commands")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let flags = result
                    .get("flags")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                println!("{}", "=".repeat(107));
                println!("| {:50} | {:50} |", "Command", "Description");
                println!("{}", "=".repeat(107));
                for item in commands {
                    let cmd = item.get("command").and_then(|v| v.as_str()).unwrap_or("");
                    let desc = item
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    println!("| {:50} | {:50} |", cmd, desc);
                    println!("{}", "-".repeat(107));
                }

                println!();

                println!("{}", "=".repeat(107));
                println!("| {:50} | {:50} |", "Flags", "Description");
                println!("{}", "-".repeat(107));
                for item in flags {
                    let flag = item.get("flag").and_then(|v| v.as_str()).unwrap_or("");
                    let desc = item
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    println!("| {:50} | {:50} |", flag, desc);
                    println!("{}", "-".repeat(107));
                }
            }
            "version" => self.print_info_cli(),
            "self-hash" => {
                let app_hash = result
                    .get("app_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");
                let cfg = result
                    .get("config_file")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".svpi");
                let cfg_hash = result.get("config_hash").and_then(|v| v.as_str());

                println!("app: {app_hash}");
                if let Some(h) = cfg_hash {
                    println!("config({cfg}): {h}");
                } else {
                    println!("config({cfg}): (not found)");
                }
            }
            "init" => {
                let memory_size = result
                    .get("memory_size")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let already_initialized = result
                    .get("already_initialized")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if already_initialized {
                    println!("Device re-initialized ({memory_size} bytes).");
                } else {
                    println!("Device initialized ({memory_size} bytes).");
                }
            }
            "check" => {
                let initialized = result
                    .get("initialized")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if initialized {
                    println!("Device is initialized.");
                } else {
                    println!("Device is not initialized.");
                }
                let architecture_ok = result
                    .get("architecture_ok")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                if !architecture_ok {
                    eprintln!("Warning: architecture mismatch.");
                }
            }
            "format" => println!("Formatted."),
            "optimize" => {
                let total = result
                    .get("memory_total")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let free = result
                    .get("memory_free")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let optimized = result
                    .get("optimized_bytes")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                println!("Optimized {optimized} bytes. Free: {free}/{total} bytes.");
            }
            "export" => {
                let file = result.get("file").and_then(|v| v.as_str()).unwrap_or("-");
                let segments = result.get("segments").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("Exported {segments} segments to '{file}'.");
            }
            "import" => {
                let file = result.get("file").and_then(|v| v.as_str()).unwrap_or("-");
                let segments = result
                    .get("segments_imported")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                println!("Imported {segments} segments from '{file}'.");
            }
            "dump" => {
                let file = result.get("file").and_then(|v| v.as_str()).unwrap_or("-");
                let bytes = result.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                let encrypted = result
                    .get("encrypted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let protection = result
                    .get("dump_protection")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let protection_label = dump_protection_label(protection);
                if encrypted {
                    println!(
                        "Dump saved to '{file}' ({bytes} bytes, encrypted: {protection_label})."
                    );
                } else {
                    println!("Dump saved to '{file}' ({bytes} bytes).");
                }
            }
            "load" => {
                let file = result.get("file").and_then(|v| v.as_str()).unwrap_or("-");
                let bytes = result.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
                let already_initialized = result
                    .get("already_initialized")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let encrypted = result
                    .get("encrypted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let protection = result
                    .get("dump_protection")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let protection_label = dump_protection_label(protection);

                if already_initialized {
                    if encrypted {
                        println!(
                            "Dump loaded from '{file}' ({bytes} bytes, encrypted: {protection_label}), existing data overwritten."
                        );
                    } else {
                        println!(
                            "Dump loaded from '{file}' ({bytes} bytes), existing data overwritten."
                        );
                    }
                } else {
                    if encrypted {
                        println!(
                            "Dump loaded from '{file}' ({bytes} bytes, encrypted: {protection_label})."
                        );
                    } else {
                        println!("Dump loaded from '{file}' ({bytes} bytes).");
                    }
                }
            }
            "set-master-password" => println!("Master password set."),
            "reset-master-password" => {
                let reset = result
                    .get("reset")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if reset {
                    println!("Master password reset.");
                } else {
                    println!("Master password is not set.");
                }
            }
            "check-master-password" => {
                let master_password_set = result
                    .get("master_password_set")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if !master_password_set {
                    println!("Master password is not set.");
                    return;
                }
                let valid = result
                    .get("valid")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if valid {
                    println!("Master password is valid.");
                } else {
                    println!("Master password is invalid.");
                }
            }
            "add-encryption-key" => {
                let name = result.get("name").and_then(|v| v.as_str()).unwrap_or("-");
                let level = result.get("level").and_then(|v| v.as_str()).unwrap_or("-");
                println!("Encryption key '{name}' added (level: {level}).");
            }
            "link-key" => {
                let name = result.get("name").and_then(|v| v.as_str()).unwrap_or("-");
                println!("Encryption key '{name}' linked.");
            }
            "sync-keys" => println!("Encryption keys synchronized."),
            "set-file" => {
                let file = result.get("file").and_then(|v| v.as_str()).unwrap_or("-");
                let cfg = result
                    .get("config_file")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".svpi");
                println!("Default vault file set to '{file}' (saved in {cfg}).");
            }
            "set" => {
                let name = result.get("name").and_then(|v| v.as_str()).unwrap_or("-");
                let data_type = result
                    .get("data_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");
                let encrypted = result
                    .get("encrypted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if encrypted {
                    println!("Saved '{name}' ({data_type}, encrypted).");
                } else {
                    println!("Saved '{name}' ({data_type}).");
                }
            }
            "get" => {
                let copied = result
                    .get("copied_to_clipboard")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if copied {
                    println!("Data copied to clipboard!");
                    return;
                }

                if let Some(data) = result.get("data").and_then(|v| v.as_str()) {
                    println!("Data: {data}");
                    return;
                }

                println!("OK");
            }
            "list" => {
                let total = result
                    .get("memory_total")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let free = result
                    .get("memory_free")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                println!("{}", "=".repeat(36));
                println!("| {:14} | {:15} |", "Memory Size", "Value (bytes)");
                println!("{}", "=".repeat(36));
                println!("| {:14} | {:15} |", "Total", total);
                println!("{}", "-".repeat(36));
                println!("| {:14} | {:15} |", "Free", free);
                println!("{}", "-".repeat(36));

                let Some(segments) = result.get("segments").and_then(|v| v.as_array()) else {
                    return;
                };

                if segments.is_empty() {
                    println!("No data found!");
                    return;
                }

                println!("{}", "=".repeat(93));
                println!(
                    "| {:32} | {:15} | {:10} | {:10} | {:10} |",
                    "Name", "Data Type", "Size", "Hash", "Pass Hash"
                );
                println!("{}", "=".repeat(93));

                let no_hash = "-".repeat(8);
                for segment in segments {
                    let name = segment.get("name").and_then(|v| v.as_str()).unwrap_or("-");
                    let data_type = segment
                        .get("data_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or(no_hash.as_str());
                    let size = segment.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                    let hash = segment
                        .get("fingerprint")
                        .and_then(|v| v.as_str())
                        .unwrap_or(no_hash.as_str());
                    let pass_hash = segment
                        .get("password_fingerprint")
                        .and_then(|v| v.as_str())
                        .unwrap_or(no_hash.as_str());
                    println!(
                        "| {:32} | {:15} | {:10} | {:10} | {:10} |",
                        name, data_type, size, hash, pass_hash
                    );
                    println!("{}", "-".repeat(93));
                }
            }
            "remove" => {
                let name = result.get("name").and_then(|v| v.as_str()).unwrap_or("-");
                println!("Removed '{name}'.");
            }
            "rename" => {
                let from = result.get("from").and_then(|v| v.as_str()).unwrap_or("-");
                let to = result.get("to").and_then(|v| v.as_str()).unwrap_or("-");
                println!("Renamed '{from}' -> '{to}'.");
            }
            "change-data-type" => {
                let name = result.get("name").and_then(|v| v.as_str()).unwrap_or("-");
                let data_type = result
                    .get("data_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-");
                println!("Data type for '{name}' changed to {data_type}.");
            }
            "change-password" => {
                let name = result.get("name").and_then(|v| v.as_str()).unwrap_or("-");
                let encrypted = result
                    .get("encrypted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if encrypted {
                    println!("Password changed for '{name}'.");
                } else {
                    println!("Encryption removed for '{name}'.");
                }
            }
            _ => {
                if let Some(data) = result.get("data").and_then(|v| v.as_str()) {
                    println!("{data}");
                } else if let Some(message) = result.get("message").and_then(|v| v.as_str()) {
                    println!("{message}");
                } else if result.is_string() || result.is_number() || result.is_boolean() {
                    println!("{result}");
                } else {
                    println!("OK");
                }
            }
        }
    }

    fn print_info_cli(&self) {
        println!("# Secure Vault Personal Information (SVPI)");
        println!("{}", "=".repeat(59));
        println!("| {:32} | {:20} |", "Info", "Value");
        println!("{}", "=".repeat(59));
        println!("| {:32} | {:>20} |", "App Version", self.meta.app_version);
        println!("{}", "-".repeat(59));
        println!(
            "| {:32} | {:20} |",
            "Architecture Version", self.meta.architecture_version
        );
        println!("{}", "-".repeat(59));
    }

    fn print_error_details_cli(details: &Value) {
        match details {
            Value::Object(map) if !map.is_empty() => {
                let is_small_flat_object = map.len() <= 6
                    && map.values().all(|v| {
                        matches!(
                            v,
                            Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null
                        )
                    });

                if is_small_flat_object {
                    for (key, value) in map {
                        match value {
                            Value::String(v) => eprintln!("{key}: {v}"),
                            Value::Number(v) => eprintln!("{key}: {v}"),
                            Value::Bool(v) => eprintln!("{key}: {v}"),
                            Value::Null => eprintln!("{key}: null"),
                            _ => {}
                        }
                    }
                    return;
                }
            }
            _ => {}
        }

        if let Ok(pretty) = serde_json::to_string_pretty(details) {
            eprintln!("{pretty}");
        }
    }
}
