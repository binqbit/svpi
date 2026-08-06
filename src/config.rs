use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use borsh::BorshDeserialize;
use borsh_derive::{BorshDeserialize, BorshSerialize};
use crate::utils::file::write_sensitive_file;

pub const CONFIG_FILE_NAME: &str = ".svpi";

const CONFIG_MAGIC: [u8; 4] = *b"SCFG";
const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SvpiConfig {
    magic: [u8; 4],
    version: u32,
    /// 0=cli, 1=json, 2=server, 3=chrome
    pub mode: u8,
    pub file: Option<String>,
}

impl Default for SvpiConfig {
    fn default() -> Self {
        Self {
            magic: CONFIG_MAGIC,
            version: CONFIG_VERSION,
            mode: 0,
            file: None,
        }
    }
}

impl SvpiConfig {
    pub fn path_in_cwd() -> io::Result<PathBuf> {
        Ok(std::env::current_dir()?.join(CONFIG_FILE_NAME))
    }

    pub fn load_from_cwd() -> io::Result<Option<Self>> {
        let path = Self::path_in_cwd()?;
        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &Path) -> io::Result<Option<Self>> {
        let bytes = match fs::read(path) {
            Ok(v) => v,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err),
        };

        // Backward-compatible: old configs can be a single byte (mode only).
        if bytes.len() == 1 {
            let mut cfg = Self::default();
            cfg.mode = bytes[0];
            return Ok(Some(cfg));
        }

        let cfg = match Self::try_from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        if (cfg.magic != CONFIG_MAGIC) || cfg.version != CONFIG_VERSION {
            return Ok(None);
        }

        Ok(Some(cfg))
    }

    pub fn save_to_cwd(&self) -> io::Result<()> {
        let path = Self::path_in_cwd()?;
        self.save_to_path(&path)
    }

    pub fn save_to_path(&self, path: &Path) -> io::Result<()> {
        let mut cfg = self.clone();
        cfg.magic = CONFIG_MAGIC;
        cfg.version = CONFIG_VERSION;

        let bytes = borsh::to_vec(&cfg)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Failed to serialize config"))?;
        write_sensitive_file(path, &bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn save_to_path_tightens_permissions_on_unix() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("svpi-config-test-{unique}.svpi"));

        let _ = fs::remove_file(&path);

        let mut cfg = SvpiConfig::default();
        cfg.mode = 1;
        cfg.file = Some("vault.bin".to_string());
        cfg.save_to_path(&path).expect("save_to_path");

        let meta = fs::metadata(&path).expect("metadata");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = meta.permissions().mode() & 0o777;
            assert_eq!(mode & 0o077, 0, "config perms must not grant group/other access");
        }

        let _ = fs::remove_file(&path);
    }
}
