use std::{
    fs,
    io::{self, Write},
    path::Path,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

const WRITE_ATTEMPTS: usize = 64;
const MAX_TMP_MODE: u32 = 0o600;

fn validate_target(path: &Path) -> io::Result<(&Path, &std::ffi::OsStr)> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path")
    })?;
    if !parent.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid parent path",
        ));
    }

    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path")
    })?;

    Ok((parent, file_name))
}

fn temp_path(
    parent: &Path,
    file_name: &std::ffi::OsStr,
    attempt: usize,
    pid: u32,
    nanos: u128,
) -> std::path::PathBuf {
    let tmp_name = format!("{}.tmp.{pid}.{nanos}.{attempt}", file_name.to_string_lossy());
    parent.join(tmp_name)
}

fn cleanup_temp_file(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(unix)]
pub(crate) fn enforce_private_permissions(path: &Path) -> io::Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    if perms.mode() & 0o777 == MAX_TMP_MODE {
        return Ok(());
    }

    perms.set_mode(MAX_TMP_MODE);
    fs::set_permissions(path, perms)?;

    let updated = fs::metadata(path)?.permissions().mode() & 0o777;
    if updated != MAX_TMP_MODE {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Failed to enforce strict file permissions",
        ));
    }

    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn enforce_private_permissions(_: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> io::Result<()> {
    let dir = fs::File::open(path)?;
    dir.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_: &Path) -> io::Result<()> {
    Ok(())
}

pub(crate) fn write_sensitive_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let (parent, file_name) = validate_target(path)?;

    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Refusing to write through symlink",
            ));
        }
    }

    let pid = process::id();
    let mut last_err: Option<io::Error> = None;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "time unavailable"))?
        .as_nanos();

    for attempt in 0..WRITE_ATTEMPTS {
        let tmp_path = temp_path(parent, file_name, attempt, pid, nanos);

        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            options.mode(MAX_TMP_MODE);
        }

        let mut file = match options.open(&tmp_path) {
            Ok(v) => v,
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                last_err = Some(err);
                continue;
            }
            Err(err) => return Err(err),
        };

        if let Err(err) = file.write_all(bytes) {
            cleanup_temp_file(&tmp_path);
            return Err(err);
        }

        #[cfg(unix)]
        {
            if let Err(err) = file.sync_data() {
                cleanup_temp_file(&tmp_path);
                return Err(err);
            }

            if let Err(err) = enforce_private_permissions(&tmp_path) {
                cleanup_temp_file(&tmp_path);
                return Err(err);
            }
        }
        #[cfg(not(unix))]
        {
            if let Err(err) = file.sync_all() {
                cleanup_temp_file(&tmp_path);
                return Err(err);
            }
        }

        match fs::rename(&tmp_path, path) {
            Ok(()) => {
                enforce_private_permissions(path)?;
                let _ = sync_directory(parent);
                return Ok(());
            }
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                if let Err(remove_err) = fs::remove_file(path) {
                    cleanup_temp_file(&tmp_path);
                    return Err(remove_err);
                }
                if let Err(err2) = fs::rename(&tmp_path, path) {
                    cleanup_temp_file(&tmp_path);
                    return Err(err2);
                }
                if let Err(err) = enforce_private_permissions(path) {
                    cleanup_temp_file(path);
                    return Err(err);
                }
                let _ = sync_directory(parent);
                return Ok(());
            }
            Err(err) => {
                cleanup_temp_file(&tmp_path);
                return Err(err);
            }
        }
    }

    match last_err {
        Some(err) => Err(err),
        None => Err(io::Error::new(io::ErrorKind::Other, "failed to create temp file")),
    }
}
