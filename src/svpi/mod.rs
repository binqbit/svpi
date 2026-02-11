pub mod cli_mode;

pub const HELP_COMMANDS: &[(&str, &str)] = &[
    ("svpi", "Start interactive command prompt (CLI mode)"),
    ("svpi config / cfg", "Print .svpi config settings"),
    (
        "svpi set-file / sf <file_name>",
        "Set default --file via .svpi",
    ),
    (
        "svpi init / i <memory_size> [low|medium|strong|hardened]",
        "Initialize the device for the desired architecture",
    ),
    ("svpi check / c", "Check the status of the device"),
    ("svpi format / f", "Format the data in the device"),
    ("svpi optimize / o", "Optimize the memory"),
    (
        "svpi resize / rs [memory_size]",
        "Resize the vault (omit size to pack to minimum)",
    ),
    ("svpi export / e <file_name>", "Export data to a file"),
    ("svpi import / m <file_name>", "Import data from a file"),
    (
        "svpi dump / d <file_name> [low|medium|strong|hardened]",
        "Dump the data from the device to a file (optionally encrypted)",
    ),
    (
        "svpi load / ld <file_name>",
        "Load dump data from a file to the device (prompts for password if encrypted)",
    ),
    (
        "svpi set-master-password / set-master",
        "Set master password",
    ),
    (
        "svpi reset-master-password / reset-master",
        "Reset master password",
    ),
    (
        "svpi check-master-password / check-master",
        "Check master password",
    ),
    (
        "svpi add-encryption-key / add-key <name>",
        "Add encryption key",
    ),
    ("svpi link-key / link <name>", "Link encryption key"),
    (
        "svpi sync-keys / sync",
        "Synchronize encryption keys fingerprints",
    ),
    ("svpi list / l", "Print all data list"),
    (
        "svpi set / s <name> <data>",
        "Set data (string, JSON byte array or binary from file)",
    ),
    ("svpi get / g <name>", "Get data"),
    ("svpi remove / r <name>", "Remove data"),
    ("svpi rename / rn <old_name> <new_name>", "Rename data"),
    (
        "svpi change-data-type / cdt <name> <new_data_type>",
        "Change data type",
    ),
    (
        "svpi change-password / cp <name>",
        "Change data password (omit new password to remove encryption)",
    ),
    (
        "svpi self-hash / hash",
        "Print SHA-256 of the running executable",
    ),
    ("svpi version / v", "Print the version of the application"),
    ("svpi help / h", "Print this help message"),
    ("svpi --mode=server", "Start the API server"),
    ("svpi --mode=chrome", "Start the Chrome app"),
];

pub const HELP_FLAGS: &[(&str, &str)] = &[
    (
        "svpi --mode=<cli|json|server|chrome>",
        "Select application mode (default: cli)",
    ),
    (
        "svpi <command> --confirm",
        "Confirm destructive actions (required in --mode=json)",
    ),
    ("svpi get --clipboard / -c", "Copy data to clipboard"),
    (
        "svpi get <name> --password=<password>",
        "Provide password via command line",
    ),
    (
        "svpi set <name> <value> --password=<password>",
        "Provide password via command line",
    ),
    (
        "svpi dump <file_name> [low|medium|strong|hardened] --password=<password>",
        "Encrypt dump with a password",
    ),
    (
        "svpi load <file_name> --password=<password>",
        "Provide password for encrypted dump",
    ),
    (
        "svpi change-password <name> --old-password=<old_password> --new-password=<new_password>",
        "Provide old and new passwords via command line",
    ),
    (
        "svpi set-master --master-password=<password>",
        "Provide master password (required in --mode=json)",
    ),
    (
        "svpi add-key <name> --master-password=<p> --key-password=<p>",
        "Provide master/key passwords (required in --mode=json)",
    ),
    (
        "svpi change-password <name> --old-password=<old_password>",
        "Remove encryption (provide only --old-password)",
    ),
    (
        "svpi --mode=server --auto-exit",
        "Automatically exit the API server after device disconnection",
    ),
    (
        "svpi --mode=server --bind=<ip> --port=<port>",
        "Configure API server bind/port (default: 127.0.0.1:3333)",
    ),
    (
        "svpi --mode=server --cors=<none|allow-all>",
        "Configure API server CORS (default: none)",
    ),
    (
        "svpi <command> --file=<file_name>",
        "Open a file password storage",
    ),
];
