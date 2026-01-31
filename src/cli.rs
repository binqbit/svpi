use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::{
    data_mgr::DataInterfaceType,
    seg_mgr::{DataType, EncryptionLevel},
    utils::response::OutputFormat,
};

/// Secure Vault Personal Information (SVPI)
#[derive(Parser, Debug)]
#[command(
    name = "svpi",
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    after_help = "Examples:\n  svpi --mode=cli list\n  svpi --mode=json get my-secret --password=123\n  svpi --mode=server --auto-exit\n\nNote: flags like --mode and --file require '=' (e.g. --mode=json)."
)]
pub struct CliArgs {
    #[arg(
        long = "mode",
        value_enum,
        default_value_t = Mode::Cli,
        global = true,
        require_equals = true,
        value_name = "MODE",
        help = "Select application mode"
    )]
    pub mode: Mode,

    #[arg(
        long = "file",
        global = true,
        require_equals = true,
        value_name = "FILE",
        help = "Use a file password storage instead of a device"
    )]
    pub file: Option<String>,

    #[arg(
        long = "confirm",
        global = true,
        help = "Confirm destructive actions (required in --mode=json)"
    )]
    pub confirm: bool,

    #[arg(
        long = "auto-exit",
        global = true,
        help = "Exit API server after device disconnection (server mode)"
    )]
    pub auto_exit: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

impl CliArgs {
    pub fn output_format(&self) -> OutputFormat {
        match self.mode {
            Mode::Json => OutputFormat::Json,
            Mode::Cli | Mode::Server | Mode::Chrome => OutputFormat::Cli,
        }
    }

    pub fn interface_type(&self) -> DataInterfaceType {
        self.file
            .as_ref()
            .map(|path| DataInterfaceType::FileSystem(path.clone()))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// Interactive CLI (default)
    Cli,
    /// JSON responses, no interactive prompts
    Json,
    /// HTTP API server (Rocket)
    Server,
    /// Chrome Native Messaging app
    Chrome,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    #[command(name = "help", alias = "h", about = "Print help message")]
    Help,

    #[command(name = "version", alias = "v", about = "Print application version")]
    Version,

    #[command(name = "init", alias = "i", about = "Initialize the device memory")]
    Init {
        #[arg(value_name = "MEMORY_SIZE", help = "Device memory size in bytes")]
        memory_size: u32,

        #[arg(
            value_enum,
            default_value_t = EncryptionLevelArg::Medium,
            value_name = "PROTECTION",
            help = "Dump protection level: low, medium, strong, hardened"
        )]
        protection: EncryptionLevelArg,
    },

    #[command(name = "check", alias = "c", about = "Check device status")]
    Check,

    #[command(name = "format", alias = "f", about = "Format device data")]
    Format,

    #[command(name = "optimize", alias = "o", about = "Optimize memory usage")]
    Optimize,

    #[command(name = "export", alias = "e", about = "Export data to a file")]
    Export {
        #[arg(value_name = "FILE", help = "Output file path")]
        file_name: String,
    },

    #[command(name = "import", alias = "m", about = "Import data from a file")]
    Import {
        #[arg(value_name = "FILE", help = "Input file path")]
        file_name: String,
    },

    #[command(
        name = "dump",
        alias = "d",
        about = "Dump raw device memory to a file (optionally encrypted)"
    )]
    Dump {
        #[arg(value_name = "FILE", help = "Output file path")]
        file_name: String,

        #[arg(
            value_enum,
            default_value_t = EncryptionLevelArg::Medium,
            value_name = "PROTECTION",
            help = "Dump encryption level: low=1, medium=2, strong=3, hardened=4"
        )]
        protection: EncryptionLevelArg,

        #[arg(
            long = "password",
            require_equals = true,
            value_name = "PASSWORD",
            help = "Password to encrypt the dump (optional in CLI; required in --mode=json to encrypt)"
        )]
        password: Option<String>,
    },

    #[command(
        name = "load",
        alias = "ld",
        about = "Load raw device dump from a file (prompts for password if encrypted)"
    )]
    Load {
        #[arg(value_name = "FILE", help = "Input file path")]
        file_name: String,

        #[arg(
            long = "password",
            require_equals = true,
            value_name = "PASSWORD",
            help = "Password to decrypt the dump (required in --mode=json if dump is encrypted)"
        )]
        password: Option<String>,
    },

    #[command(
        name = "set-master-password",
        alias = "set-master",
        about = "Set master password"
    )]
    SetMasterPassword(SetMasterPasswordArgs),

    #[command(
        name = "reset-master-password",
        alias = "reset-master",
        about = "Reset master password"
    )]
    ResetMasterPassword,

    #[command(
        name = "check-master-password",
        alias = "check-master",
        about = "Check master password"
    )]
    CheckMasterPassword(SetMasterPasswordArgs),

    #[command(
        name = "add-encryption-key",
        alias = "add-key",
        about = "Add encryption key"
    )]
    AddEncryptionKey(AddEncryptionKeyArgs),

    #[command(name = "link-key", alias = "link", about = "Link encryption key")]
    LinkKey(LinkKeyArgs),

    #[command(
        name = "sync-keys",
        alias = "sync",
        about = "Synchronize encryption keys fingerprints"
    )]
    SyncKeys(SyncKeysArgs),

    #[command(name = "list", alias = "l", about = "Print all data list")]
    List,

    #[command(name = "set", alias = "s", about = "Set data")]
    Set(SetArgs),

    #[command(name = "get", alias = "g", about = "Get data")]
    Get(GetArgs),

    #[command(name = "remove", alias = "r", about = "Remove data")]
    Remove { name: String },

    #[command(name = "rename", alias = "rn", about = "Rename data")]
    Rename { old_name: String, new_name: String },

    #[command(name = "change-data-type", alias = "cdt", about = "Change data type")]
    ChangeDataType(ChangeDataTypeArgs),

    #[command(name = "change-password", alias = "cp", about = "Change data password")]
    ChangePassword(ChangePasswordArgs),
}

#[derive(Debug, Clone, Args)]
pub struct SetMasterPasswordArgs {
    #[arg(
        long = "master-password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Provide master password via command line"
    )]
    pub master_password: Option<String>,

    #[arg(
        long = "password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Password fallback (used if --master-password is not set)"
    )]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EncryptionLevelArg {
    /// Faster, weaker
    Low,
    /// Balanced (default)
    Medium,
    /// Slower, stronger
    Strong,
    /// Slowest, strongest
    Hardened,
}

impl EncryptionLevelArg {
    pub const fn as_str(self) -> &'static str {
        match self {
            EncryptionLevelArg::Low => "low",
            EncryptionLevelArg::Medium => "medium",
            EncryptionLevelArg::Strong => "strong",
            EncryptionLevelArg::Hardened => "hardened",
        }
    }

    pub const fn multiplier(self) -> u32 {
        match self {
            EncryptionLevelArg::Low => 1,
            EncryptionLevelArg::Medium => 2,
            EncryptionLevelArg::Strong => 4,
            EncryptionLevelArg::Hardened => 4,
        }
    }
}

impl From<EncryptionLevelArg> for EncryptionLevel {
    fn from(level: EncryptionLevelArg) -> Self {
        match level {
            EncryptionLevelArg::Low => EncryptionLevel::Low,
            EncryptionLevelArg::Medium => EncryptionLevel::Medium,
            EncryptionLevelArg::Strong => EncryptionLevel::Strong,
            EncryptionLevelArg::Hardened => EncryptionLevel::Hardened,
        }
    }
}

#[derive(Debug, Clone, Args)]
pub struct AddEncryptionKeyArgs {
    #[arg(value_name = "NAME", help = "Encryption key name")]
    pub name: String,

    #[arg(
        value_enum,
        default_value_t = EncryptionLevelArg::Medium,
        value_name = "LEVEL",
        help = "Encryption level (default: medium)"
    )]
    pub level: EncryptionLevelArg,

    #[arg(
        long = "master-password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Master password (required in --mode=json if master password is set)"
    )]
    pub master_password: Option<String>,

    #[arg(
        long = "key-password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Encryption key password (required in --mode=json)"
    )]
    pub key_password: Option<String>,

    #[arg(
        long = "password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Password fallback for master/key passwords"
    )]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct LinkKeyArgs {
    #[arg(value_name = "NAME", help = "Encryption key name")]
    pub name: String,

    #[arg(
        long = "password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Encryption key password"
    )]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct SyncKeysArgs {
    #[arg(
        long = "master-password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Master password (required in --mode=json if master password is set)"
    )]
    pub master_password: Option<String>,

    #[arg(
        long = "password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Password fallback (used if --master-password is not set)"
    )]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct SetArgs {
    #[arg(value_name = "NAME", help = "Entry name")]
    pub name: String,

    #[arg(
        value_name = "DATA",
        help = "Data value (JSON byte array or binary from file path allowed; optional in CLI: reads clipboard if omitted; required in --mode=json)"
    )]
    pub data: Option<String>,

    #[arg(
        long = "password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Password to encrypt data"
    )]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct GetArgs {
    #[arg(value_name = "NAME", help = "Entry name")]
    pub name: String,

    #[arg(
        long = "password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Password for decryption"
    )]
    pub password: Option<String>,

    #[arg(long = "clipboard", short = 'c', help = "Copy data to clipboard")]
    pub clipboard: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DataTypeArg {
    Binary,
    Plain,
    Hex,
    Base58,
    Base64,
}

impl From<DataTypeArg> for DataType {
    fn from(value: DataTypeArg) -> Self {
        match value {
            DataTypeArg::Binary => DataType::Binary,
            DataTypeArg::Plain => DataType::Plain,
            DataTypeArg::Hex => DataType::Hex,
            DataTypeArg::Base58 => DataType::Base58,
            DataTypeArg::Base64 => DataType::Base64,
        }
    }
}

#[derive(Debug, Clone, Args)]
pub struct ChangeDataTypeArgs {
    #[arg(value_name = "NAME", help = "Entry name")]
    pub name: String,

    #[arg(value_enum, value_name = "DATA_TYPE", help = "New data type")]
    pub new_data_type: DataTypeArg,
}

#[derive(Debug, Clone, Args)]
pub struct ChangePasswordArgs {
    #[arg(value_name = "NAME", help = "Entry name")]
    pub name: String,

    #[arg(
        long = "old-password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "Old password (required if entry is encrypted)"
    )]
    pub old_password: Option<String>,

    #[arg(
        long = "new-password",
        require_equals = true,
        value_name = "PASSWORD",
        help = "New password (to change; omit to remove encryption)"
    )]
    pub new_password: Option<String>,
}
