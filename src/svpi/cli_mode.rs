use std::fs;

use arboard::Clipboard;
use serde_json::{json, Value};

use crate::{
    cli,
    data_mgr::DataInterfaceType,
    pass_mgr::PasswordManager,
    protocol::segments::SegmentSummary,
    seg_mgr::{Data, DataType, FormattedData},
    utils::{
        response::{OutputFormat, SvpiResponse},
        terminal,
    },
};

fn load_mgr(
    interface_type: &DataInterfaceType,
    cmd: Option<String>,
) -> Result<PasswordManager, (SvpiResponse, i32)> {
    PasswordManager::try_load(interface_type.clone())
        .map_err(|e| SvpiResponse::data_manager_error_verbose(cmd, e).with_exit_code())
}

fn open_mgr_for_uninitialized(
    interface_type: &DataInterfaceType,
    cmd: Option<String>,
) -> Result<PasswordManager, (SvpiResponse, i32)> {
    PasswordManager::from_device_type(interface_type.clone())
        .map_err(|e| SvpiResponse::data_manager_error_verbose(cmd, e).with_exit_code())
}

fn confirm_or_require_confirm(
    confirm: bool,
    output_mode: OutputFormat,
    cmd_out: Option<String>,
    prompt: &str,
    action: &str,
    details: Value,
) -> Result<(), (SvpiResponse, i32)> {
    if confirm {
        return Ok(());
    }

    match output_mode {
        OutputFormat::Cli => {
            if terminal::confirm(prompt) {
                Ok(())
            } else {
                Err((
                    SvpiResponse::ok(
                        cmd_out,
                        json!({ "cancelled": true, "action": action, "context": details }),
                    ),
                    0,
                ))
            }
        }
        OutputFormat::Json => {
            Err(SvpiResponse::confirmation_required(cmd_out, action, details).with_exit_code())
        }
    }
}

pub fn run_with_cli(cli: &cli::CliArgs) -> i32 {
    let output_mode = cli.output_format();
    let interface_type = cli.interface_type();
    let confirm = cli.confirm;

    let command = cli.command.clone().unwrap_or(cli::Command::Help);
    let (resp, code) = execute_with_output(command, output_mode, &interface_type, confirm);

    resp.print(output_mode);

    code
}

fn command_name(cmd: &cli::Command) -> &'static str {
    match cmd {
        cli::Command::Help => "help",
        cli::Command::Version => "version",
        cli::Command::Init { .. } => "init",
        cli::Command::Check => "check",
        cli::Command::Format => "format",
        cli::Command::Optimize => "optimize",
        cli::Command::Export { .. } => "export",
        cli::Command::Import { .. } => "import",
        cli::Command::Dump { .. } => "dump",
        cli::Command::Load { .. } => "load",
        cli::Command::SetMasterPassword(_) => "set-master-password",
        cli::Command::ResetMasterPassword => "reset-master-password",
        cli::Command::CheckMasterPassword(_) => "check-master-password",
        cli::Command::AddEncryptionKey(_) => "add-encryption-key",
        cli::Command::LinkKey(_) => "link-key",
        cli::Command::SyncKeys(_) => "sync-keys",
        cli::Command::List => "list",
        cli::Command::Set(_) => "set",
        cli::Command::Get(_) => "get",
        cli::Command::Remove { .. } => "remove",
        cli::Command::Rename { .. } => "rename",
        cli::Command::ChangeDataType(_) => "change-data-type",
        cli::Command::ChangePassword(_) => "change-password",
    }
}

fn execute_with_output(
    cmd: cli::Command,
    output_mode: OutputFormat,
    interface_type: &DataInterfaceType,
    confirm: bool,
) -> (SvpiResponse, i32) {
    let cmd_str = command_name(&cmd).to_string();
    let cmd_out = Some(cmd_str.clone());

    match cmd {
        cli::Command::Help => {
            let commands = crate::svpi::HELP_COMMANDS
                .iter()
                .map(|(cmd, desc)| json!({"command": cmd, "description": desc}))
                .collect::<Vec<_>>();
            let flags = crate::svpi::HELP_FLAGS
                .iter()
                .map(|(flag, desc)| json!({"flag": flag, "description": desc}))
                .collect::<Vec<_>>();
            (
                SvpiResponse::ok(cmd_out, json!({ "commands": commands, "flags": flags })),
                0,
            )
        }
        cli::Command::Version => (
            SvpiResponse::ok(
                cmd_out,
                json!({
                    "app_version": env!("CARGO_PKG_VERSION"),
                    "architecture_version": crate::seg_mgr::ARCHITECTURE_VERSION,
                }),
            ),
            0,
        ),

        cli::Command::Init { memory_size } => {
            let mut pass_mgr = match open_mgr_for_uninitialized(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let already_initialized = match pass_mgr.get_data_manager().check_init_data() {
                Ok(v) => v,
                Err(err) => {
                    return SvpiResponse::err(cmd_out, "device_error", err.to_string(), None)
                        .with_exit_code();
                }
            };

            let prompt = if already_initialized {
                "Device already initialized. Re-initializing will erase all data. Continue?"
            } else {
                "Are you sure you want to initialize the device?"
            };
            if let Err(err) = confirm_or_require_confirm(
                confirm,
                output_mode,
                cmd_out.clone(),
                prompt,
                "init",
                json!({ "memory_size": memory_size, "already_initialized": already_initialized }),
            ) {
                return err;
            }

            if let Err(err) = pass_mgr.get_data_manager().init_device(memory_size) {
                return SvpiResponse::data_manager_error_verbose(cmd_out, err).with_exit_code();
            }

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({
                        "initialized": true,
                        "memory_size": memory_size,
                        "already_initialized": already_initialized,
                    }),
                ),
                0,
            )
        }

        cli::Command::Check => {
            let mut pass_mgr = match open_mgr_for_uninitialized(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let initialized = match pass_mgr.get_data_manager().check_init_data() {
                Ok(v) => v,
                Err(err) => {
                    return SvpiResponse::err(cmd_out, "device_error", err.to_string(), None)
                        .with_exit_code();
                }
            };
            let architecture_ok = match pass_mgr.get_data_manager().check_architecture_version() {
                Ok(v) => v,
                Err(_) => false,
            };

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({ "initialized": initialized, "architecture_ok": architecture_ok }),
                ),
                0,
            )
        }

        cli::Command::Format => {
            if let Err(err) = confirm_or_require_confirm(
                confirm,
                output_mode,
                cmd_out.clone(),
                "Are you sure you want to format the data?",
                "format",
                json!({}),
            ) {
                return err;
            }

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            if let Err(err) = pass_mgr.get_data_manager().format_data() {
                return SvpiResponse::err(cmd_out, "device_error", err.to_string(), None)
                    .with_exit_code();
            }

            (SvpiResponse::ok(cmd_out, json!({ "formatted": true })), 0)
        }

        cli::Command::Optimize => {
            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let optimized = match pass_mgr.get_data_manager().optimize_segments() {
                Ok(v) => v,
                Err(err) => {
                    return SvpiResponse::err(cmd_out, "device_error", err.to_string(), None)
                        .with_exit_code();
                }
            };

            let seg_mgr = pass_mgr.get_data_manager();
            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({
                        "memory_total": seg_mgr.metadata.memory_size,
                        "memory_free": seg_mgr.free_memory_size(),
                        "optimized_bytes": optimized,
                    }),
                ),
                0,
            )
        }

        cli::Command::Export { file_name } => {
            let file_path = file_name;

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let mut list = Vec::new();
            for seg in pass_mgr
                .get_data_manager()
                .get_active_segments_mut()
                .into_iter()
                .filter(|seg| seg.info.data_type != DataType::EncryptionKey)
            {
                let data = match seg.read_data() {
                    Ok(data) => data,
                    Err(err) => {
                        return SvpiResponse::err(cmd_out, "device_error", err.to_string(), None)
                            .with_exit_code();
                    }
                };

                let formatted = match FormattedData::new(
                    seg.get_name(),
                    data,
                    seg.info.data_type,
                    seg.info.password_fingerprint,
                )
                .encode()
                {
                    Ok(v) => v,
                    Err(err) => {
                        return SvpiResponse::err(cmd_out, "internal_error", err.to_string(), None)
                            .with_exit_code();
                    }
                };
                list.push(formatted);
            }

            if let Err(err) = fs::write(&file_path, list.join("\n")) {
                return SvpiResponse::err(cmd_out, "io_error", err.to_string(), None)
                    .with_exit_code();
            }

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({ "exported": true, "file": file_path, "segments": list.len() }),
                ),
                0,
            )
        }

        cli::Command::Import { file_name } => {
            let file_path = file_name;

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let text = match fs::read_to_string(&file_path) {
                Ok(v) => v,
                Err(err) => {
                    return SvpiResponse::err(cmd_out, "io_error", err.to_string(), None)
                        .with_exit_code();
                }
            };

            let seg_mgr = pass_mgr.get_data_manager();
            let mut imported = 0usize;
            for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
                let formatted = match FormattedData::decode(line) {
                    Ok(v) => v,
                    Err(err) => {
                        return SvpiResponse::err(
                            cmd_out,
                            "invalid_argument",
                            err.to_string(),
                            Some(json!({ "line": line })),
                        )
                        .with_exit_code();
                    }
                };

                let data = match formatted.data.to_bytes() {
                    Ok(v) => v,
                    Err(err) => {
                        return SvpiResponse::err(
                            cmd_out,
                            "invalid_argument",
                            err.to_string(),
                            Some(json!({ "name": formatted.name })),
                        )
                        .with_exit_code();
                    }
                };

                let seg = match seg_mgr.set_segment(
                    &formatted.name,
                    &data,
                    formatted.data_type,
                    formatted.password_fingerprint,
                ) {
                    Ok(v) => v,
                    Err(err) => {
                        return SvpiResponse::err(cmd_out, "device_error", err.to_string(), None)
                            .with_exit_code();
                    }
                };

                if seg.is_none() {
                    return SvpiResponse::err(
                        cmd_out,
                        "not_enough_memory",
                        "Not enough memory (try `svpi optimize`)".to_string(),
                        Some(json!({ "imported": imported })),
                    )
                    .with_exit_code();
                }

                imported += 1;
            }

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({ "imported": true, "file": file_path, "segments_imported": imported }),
                ),
                0,
            )
        }

        cli::Command::Dump { file_name } => {
            let file_path = file_name;

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let dump = match pass_mgr.get_data_manager().get_dump() {
                Ok(v) => v,
                Err(err) => {
                    return SvpiResponse::err(cmd_out, "device_error", err.to_string(), None)
                        .with_exit_code();
                }
            };
            if let Err(err) = fs::write(&file_path, &dump) {
                return SvpiResponse::err(cmd_out, "io_error", err.to_string(), None)
                    .with_exit_code();
            }

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({ "dump_saved": true, "file": file_path, "bytes": dump.len() }),
                ),
                0,
            )
        }

        cli::Command::Load { file_name } => {
            let file_path = file_name;

            let mut pass_mgr = match open_mgr_for_uninitialized(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let already_initialized = match pass_mgr.get_data_manager().check_init_data() {
                Ok(v) => v,
                Err(_) => false,
            };

            let prompt = if already_initialized {
                "Device already initialized. Loading a dump will overwrite data. Continue?"
            } else {
                "Are you sure you want to load the dump?"
            };
            if let Err(err) = confirm_or_require_confirm(
                confirm,
                output_mode,
                cmd_out.clone(),
                prompt,
                "load_dump",
                json!({ "file": file_path, "already_initialized": already_initialized }),
            ) {
                return err;
            }

            let dump = match fs::read(&file_path) {
                Ok(v) => v,
                Err(err) => {
                    return SvpiResponse::err(cmd_out, "io_error", err.to_string(), None)
                        .with_exit_code();
                }
            };

            if let Err(err) = pass_mgr.get_data_manager().set_dump(&dump) {
                return SvpiResponse::err(cmd_out, "device_error", err.to_string(), None)
                    .with_exit_code();
            }

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({
                        "dump_loaded": true,
                        "file": file_path,
                        "bytes": dump.len(),
                        "already_initialized": already_initialized,
                    }),
                ),
                0,
            )
        }

        cli::Command::SetMasterPassword(args) => {
            let mut master_password = args
                .master_password
                .or(args.password)
                .filter(|p| !p.is_empty());
            if master_password.is_none() && output_mode == OutputFormat::Cli {
                master_password = terminal::get_password_confirmed(Some("master password"))
                    .or_else(|| {
                        let mut clipboard = Clipboard::new().ok()?;
                        clipboard.get_text().ok().filter(|t| !t.trim().is_empty())
                    });
            }
            let Some(master_password) = master_password else {
                if output_mode == OutputFormat::Cli {
                    return SvpiResponse::cancelled(cmd_out, "set-master-password", json!({}))
                        .with_exit_code();
                }
                return SvpiResponse::missing_argument(
                    cmd_out,
                    "master_password (--master-password=...)",
                )
                .with_exit_code();
            };

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            if let Err(err) = pass_mgr.set_master_password(&master_password) {
                return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code();
            }

            (
                SvpiResponse::ok(cmd_out, json!({ "master_password_set": true })),
                0,
            )
        }

        cli::Command::ResetMasterPassword => {
            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            if !pass_mgr.is_master_password_set() {
                return (
                    SvpiResponse::ok(
                        cmd_out,
                        json!({ "master_password_set": false, "reset": false }),
                    ),
                    0,
                );
            }

            if let Err(err) = confirm_or_require_confirm(
                confirm,
                output_mode,
                cmd_out.clone(),
                "Are you sure you want to remove the master password?",
                "reset_master_password",
                json!({}),
            ) {
                return err;
            }

            if let Err(err) = pass_mgr.reset_master_password() {
                return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code();
            }

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({ "master_password_set": true, "reset": true }),
                ),
                0,
            )
        }

        cli::Command::CheckMasterPassword(args) => {
            let pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            if !pass_mgr.is_master_password_set() {
                return (
                    SvpiResponse::ok(cmd_out, json!({ "master_password_set": false })),
                    0,
                );
            }

            let mut master_password = args
                .master_password
                .or(args.password)
                .filter(|p| !p.is_empty());
            if master_password.is_none() && output_mode == OutputFormat::Cli {
                master_password = terminal::get_password(Some("master password")).or_else(|| {
                    let mut clipboard = Clipboard::new().ok()?;
                    clipboard.get_text().ok().filter(|t| !t.trim().is_empty())
                });
            }
            let Some(master_password) = master_password else {
                if output_mode == OutputFormat::Cli {
                    return SvpiResponse::cancelled(cmd_out, "check-master-password", json!({}))
                        .with_exit_code();
                }
                return SvpiResponse::missing_argument(
                    cmd_out,
                    "master_password (--master-password=...)",
                )
                .with_exit_code();
            };

            let valid = pass_mgr.check_master_password(&master_password);
            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({ "master_password_set": true, "valid": valid }),
                ),
                0,
            )
        }

        cli::Command::AddEncryptionKey(args) => {
            let name = args.name;
            let level: crate::seg_mgr::EncryptionLevel = args.level.into();

            let mut master_password = args
                .master_password
                .or_else(|| args.password.clone())
                .filter(|p| !p.is_empty());
            if master_password.is_none() && output_mode == OutputFormat::Cli {
                master_password = terminal::get_password_confirmed(Some("master password"))
                    .or_else(|| {
                        let mut clipboard = Clipboard::new().ok()?;
                        clipboard.get_text().ok().filter(|t| !t.trim().is_empty())
                    });
            }
            let Some(master_password) = master_password else {
                if output_mode == OutputFormat::Cli {
                    return SvpiResponse::cancelled(
                        cmd_out,
                        "add-encryption-key",
                        json!({ "name": name }),
                    )
                    .with_exit_code();
                }
                return SvpiResponse::missing_argument(
                    cmd_out,
                    "master_password (--master-password=...)",
                )
                .with_exit_code();
            };

            let mut key_password = args
                .key_password
                .or(args.password)
                .filter(|p| !p.is_empty());
            if key_password.is_none() && output_mode == OutputFormat::Cli {
                key_password = terminal::get_password_confirmed(Some("key password"));
            }
            let Some(key_password) = key_password else {
                if output_mode == OutputFormat::Cli {
                    return SvpiResponse::cancelled(
                        cmd_out,
                        "add-encryption-key",
                        json!({ "name": name }),
                    )
                    .with_exit_code();
                }
                return SvpiResponse::missing_argument(
                    cmd_out,
                    "key_password (--key-password=...)",
                )
                .with_exit_code();
            };

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            if !pass_mgr.is_master_password_set() {
                return SvpiResponse::err(
                    cmd_out,
                    "master_password_not_set",
                    "Master password not set",
                    None,
                )
                .with_exit_code();
            }
            if !pass_mgr.check_master_password(&master_password) {
                return SvpiResponse::err(
                    cmd_out,
                    "master_password_invalid",
                    "Master password is invalid",
                    None,
                )
                .with_exit_code();
            }

            match pass_mgr.add_encryption_key(&master_password, &name, &key_password, level) {
                Ok(true) => (
                    SvpiResponse::ok(
                        cmd_out,
                        json!({ "added": true, "name": name, "level": format!("{level:?}").to_lowercase() }),
                    ),
                    0,
                ),
                Ok(false) => {
                    SvpiResponse::err(cmd_out, "not_enough_memory", "Not enough memory", None)
                        .with_exit_code()
                }
                Err(err) => SvpiResponse::password_manager_error(cmd_out, err).with_exit_code(),
            }
        }

        cli::Command::LinkKey(args) => {
            let name = args.name;
            let mut password = args.password.filter(|p| !p.is_empty());
            if password.is_none() && output_mode == OutputFormat::Cli {
                password = terminal::get_password_confirmed(Some("password"));
            }
            let Some(password) = password else {
                if output_mode == OutputFormat::Cli {
                    return SvpiResponse::cancelled(cmd_out, "link-key", json!({ "name": name }))
                        .with_exit_code();
                }
                return SvpiResponse::missing_argument(cmd_out, "password (--password=...)")
                    .with_exit_code();
            };

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            if let Err(err) = pass_mgr.link_key(&name, &password) {
                return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code();
            }

            (
                SvpiResponse::ok(cmd_out, json!({ "linked": true, "name": name })),
                0,
            )
        }

        cli::Command::SyncKeys(args) => {
            let mut master_password = args
                .master_password
                .or(args.password)
                .filter(|p| !p.is_empty());
            if master_password.is_none() && output_mode == OutputFormat::Cli {
                master_password = terminal::get_password_confirmed(Some("master password"))
                    .or_else(|| {
                        let mut clipboard = Clipboard::new().ok()?;
                        clipboard.get_text().ok().filter(|t| !t.trim().is_empty())
                    });
            }
            let Some(master_password) = master_password else {
                if output_mode == OutputFormat::Cli {
                    return SvpiResponse::cancelled(cmd_out, "sync-keys", json!({}))
                        .with_exit_code();
                }
                return SvpiResponse::missing_argument(
                    cmd_out,
                    "master_password (--master-password=...)",
                )
                .with_exit_code();
            };

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            if !pass_mgr.is_master_password_set() {
                return SvpiResponse::err(
                    cmd_out,
                    "master_password_not_set",
                    "Master password not set",
                    None,
                )
                .with_exit_code();
            }
            if !pass_mgr.check_master_password(&master_password) {
                return SvpiResponse::err(
                    cmd_out,
                    "master_password_invalid",
                    "Master password is invalid",
                    None,
                )
                .with_exit_code();
            }

            if let Err(err) = pass_mgr.sync_encryption_keys(&master_password) {
                return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code();
            }

            (SvpiResponse::ok(cmd_out, json!({ "synced": true })), 0)
        }

        cli::Command::List => {
            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };
            let seg_mgr = pass_mgr.get_data_manager();

            let segments = seg_mgr
                .get_active_segments()
                .into_iter()
                .map(|seg| SegmentSummary::from_segment(&seg))
                .collect::<Vec<_>>();

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({
                        "memory_total": seg_mgr.metadata.memory_size,
                        "memory_free": seg_mgr.free_memory_size(),
                        "segments": segments,
                    }),
                ),
                0,
            )
        }

        cli::Command::Set(args) => {
            let name = args.name;
            let data = match args.data {
                Some(v) => v,
                None if output_mode == OutputFormat::Cli => {
                    let mut clipboard = match Clipboard::new() {
                        Ok(v) => v,
                        Err(err) => {
                            return SvpiResponse::err(
                                cmd_out,
                                "clipboard_error",
                                err.to_string(),
                                None,
                            )
                            .with_exit_code();
                        }
                    };
                    match clipboard.get_text() {
                        Ok(v) if !v.trim().is_empty() => v,
                        Ok(_) => {
                            return SvpiResponse::missing_argument(cmd_out, "data").with_exit_code()
                        }
                        Err(err) => {
                            return SvpiResponse::err(
                                cmd_out,
                                "clipboard_error",
                                err.to_string(),
                                None,
                            )
                            .with_exit_code();
                        }
                    }
                }
                None => return SvpiResponse::missing_argument(cmd_out, "data").with_exit_code(),
            };
            let encryption_key = args.password.filter(|p| !p.is_empty()).or_else(|| {
                if output_mode == OutputFormat::Cli {
                    terminal::get_password_confirmed(None)
                } else {
                    None
                }
            });

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let saved = match pass_mgr.save_password(&name, &data, encryption_key.clone()) {
                Ok(v) => v,
                Err(err) => {
                    return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code()
                }
            };

            if !saved {
                return SvpiResponse::err(cmd_out, "not_enough_memory", "Not enough memory", None)
                    .with_exit_code();
            }

            let inferred = Data::from_str_infer(&data);
            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({
                        "saved": true,
                        "name": name,
                        "data_type": inferred.get_type().to_string(),
                        "encrypted": encryption_key.is_some(),
                    }),
                ),
                0,
            )
        }

        cli::Command::Get(args) => {
            let name = args.name;
            let mut password = args.password.filter(|p| !p.is_empty());
            let to_clipboard = args.clipboard;

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let (encrypted, data_type) = {
                let seg = pass_mgr.get_data_manager().find_segment_by_name(&name);
                let Some(seg) = seg else {
                    return SvpiResponse::err(
                        cmd_out,
                        "data_not_found",
                        format!("Data '{name}' not found"),
                        None,
                    )
                    .with_exit_code();
                };
                (seg.info.password_fingerprint.is_some(), seg.info.data_type)
            };

            if encrypted && password.is_none() && output_mode == OutputFormat::Cli {
                password = terminal::get_password(None);
                if password.is_none() {
                    return SvpiResponse::cancelled(cmd_out, "get", json!({ "name": name }))
                        .with_exit_code();
                }
            }
            if encrypted && password.is_none() {
                return SvpiResponse::err(
                    cmd_out,
                    "password_required",
                    "Password required for decryption (pass --password=...)",
                    Some(json!({ "name": name })),
                )
                .with_exit_code();
            }

            let data = match pass_mgr.read_password(&name, || password.clone().unwrap_or_default())
            {
                Ok(v) => v,
                Err(err) => {
                    return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code()
                }
            };

            if to_clipboard {
                let mut clipboard = match Clipboard::new() {
                    Ok(v) => v,
                    Err(err) => {
                        return SvpiResponse::err(
                            cmd_out,
                            "clipboard_error",
                            err.to_string(),
                            None,
                        )
                        .with_exit_code();
                    }
                };
                if let Err(err) = clipboard.set_text(data) {
                    return SvpiResponse::err(cmd_out, "clipboard_error", err.to_string(), None)
                        .with_exit_code();
                }
                return (
                    SvpiResponse::ok(
                        cmd_out,
                        json!({ "name": name, "copied_to_clipboard": true, "data_type": data_type.to_string(), "encrypted": encrypted }),
                    ),
                    0,
                );
            }

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({ "name": name, "data": data, "data_type": data_type.to_string(), "encrypted": encrypted }),
                ),
                0,
            )
        }

        cli::Command::Remove { name } => {
            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let exists = pass_mgr
                .get_data_manager()
                .find_segment_by_name(&name)
                .is_some();
            if !exists {
                return SvpiResponse::err(
                    cmd_out,
                    "data_not_found",
                    format!("Data '{name}' not found"),
                    None,
                )
                .with_exit_code();
            }

            let prompt = format!("Are you sure you want to remove '{name}'?");
            if let Err(err) = confirm_or_require_confirm(
                confirm,
                output_mode,
                cmd_out.clone(),
                &prompt,
                "remove",
                json!({ "name": name }),
            ) {
                return err;
            }

            if let Err(err) = pass_mgr.remove_password(&name) {
                return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code();
            }

            (
                SvpiResponse::ok(cmd_out, json!({ "removed": true, "name": name })),
                0,
            )
        }

        cli::Command::Rename { old_name, new_name } => {
            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let exists = pass_mgr
                .get_data_manager()
                .find_segment_by_name(&old_name)
                .is_some();
            if !exists {
                return SvpiResponse::err(
                    cmd_out,
                    "data_not_found",
                    format!("Data '{old_name}' not found"),
                    None,
                )
                .with_exit_code();
            }

            if let Err(err) = pass_mgr.rename_password(&old_name, &new_name) {
                return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code();
            }

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({ "renamed": true, "from": old_name, "to": new_name }),
                ),
                0,
            )
        }

        cli::Command::ChangeDataType(args) => {
            let name = args.name;
            let new_type: DataType = args.new_data_type.into();

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let exists = pass_mgr
                .get_data_manager()
                .find_segment_by_name(&name)
                .is_some();
            if !exists {
                return SvpiResponse::err(
                    cmd_out,
                    "data_not_found",
                    format!("Data '{name}' not found"),
                    None,
                )
                .with_exit_code();
            }

            if let Err(err) = pass_mgr.change_data_type(&name, new_type) {
                return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code();
            }

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({ "changed": true, "name": name, "data_type": new_type.to_string() }),
                ),
                0,
            )
        }

        cli::Command::ChangePassword(args) => {
            let name = args.name;

            let mut old_password = args.old_password.filter(|p| !p.is_empty());
            let mut new_password = args.new_password.filter(|p| !p.is_empty());

            let mut pass_mgr = match load_mgr(interface_type, cmd_out.clone()) {
                Ok(mgr) => mgr,
                Err(err) => return err,
            };

            let encrypted = {
                let seg = pass_mgr.get_data_manager().find_segment_by_name(&name);
                let Some(seg) = seg else {
                    return SvpiResponse::err(
                        cmd_out,
                        "data_not_found",
                        format!("Data '{name}' not found"),
                        None,
                    )
                    .with_exit_code();
                };
                seg.info.password_fingerprint.is_some()
            };

            if encrypted && old_password.is_none() && output_mode == OutputFormat::Cli {
                old_password = terminal::get_password(Some("old password"));
                if old_password.is_none() {
                    return SvpiResponse::cancelled(
                        cmd_out,
                        "change-password",
                        json!({ "name": name }),
                    )
                    .with_exit_code();
                }
            }
            if encrypted && old_password.is_none() {
                return SvpiResponse::err(
                    cmd_out,
                    "password_required",
                    "Old password required (pass --old-password=...)",
                    Some(json!({ "name": name })),
                )
                .with_exit_code();
            }

            if new_password.is_none() && output_mode == OutputFormat::Cli {
                new_password = terminal::get_password_confirmed(Some("new password"));
            }

            let data =
                match pass_mgr.read_password(&name, || old_password.clone().unwrap_or_default()) {
                    Ok(v) => v,
                    Err(err) => {
                        return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code()
                    }
                };

            let saved = match pass_mgr.save_password(&name, &data, new_password.clone()) {
                Ok(v) => v,
                Err(err) => {
                    return SvpiResponse::password_manager_error(cmd_out, err).with_exit_code()
                }
            };
            if !saved {
                return SvpiResponse::err(cmd_out, "not_enough_memory", "Not enough memory", None)
                    .with_exit_code();
            }

            (
                SvpiResponse::ok(
                    cmd_out,
                    json!({ "changed": true, "name": name, "encrypted": new_password.is_some() }),
                ),
                0,
            )
        }
    }
}
