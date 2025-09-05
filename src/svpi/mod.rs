pub mod data;
pub mod device;
pub mod password;

pub const HELP_COMMANDS: &[(&str, &str)] = &[
    (
        "svpi init / i <memory_size>",
        "Initialize the device for the desired architecture",
    ),
    ("svpi check / c", "Check the status of the device"),
    ("svpi format / f", "Format the data in the device"),
    ("svpi optimize / o", "Optimize the memory"),
    ("svpi export / e <file_name>", "Export data to a file"),
    ("svpi import / m <file_name>", "Import data from a file"),
    (
        "svpi dump / d <file_name>",
        "Dump the data from the device to a file",
    ),
    (
        "svpi load / ld <file_name>",
        "Load dump data from a file to the device",
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
    ("svpi list / l", "Print all data list"),
    ("svpi set / s <name> <data>", "Set data"),
    ("svpi get / g <name>", "Get data"),
    ("svpi remove / r <name>", "Remove data"),
    ("svpi rename / rn <old_name> <new_name>", "Rename data"),
    (
        "svpi change-data-type / cdt <name> <new_data_type>",
        "Change data type",
    ),
    ("svpi change-password / cp <name>", "Change data password"),
    ("svpi version / v", "Print the version of the application"),
    ("svpi help / h", "Print this help message"),
    ("svpi api-server", "Start the API server"),
    ("svpi api-chrome", "Start the Chrome app"),
];

pub const HELP_FLAGS: &[(&str, &str)] = &[
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
        "svpi change-password <name> --old-password=<old_password> --new-password=<new_password>",
        "Provide old and new passwords via command line",
    ),
    (
        "svpi api-server --auto-exit / -ae",
        "Automatically exit the API server after device disconnection",
    ),
    (
        "svpi <command> --file=<file_name>",
        "Open a file password storage",
    ),
];
