use std::{
    fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use borsh::BorshDeserialize;
use borsh_derive::{BorshDeserialize, BorshSerialize};

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

        let Some(parent) = path.parent() else {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "Invalid config path",
            ));
        };
        let tmp_path = parent.join(format!("{CONFIG_FILE_NAME}.tmp"));

        fs::write(&tmp_path, bytes)?;
        if let Err(err) = fs::rename(&tmp_path, path) {
            if err.kind() != ErrorKind::AlreadyExists {
                return Err(err);
            }
            let _ = fs::remove_file(path);
            fs::rename(&tmp_path, path)?;
        }

        Ok(())
    }
}
