
mod spdm;
mod seg_mgmt;
mod svpi;
mod utils;

use arboard::Clipboard;
use spdm::SerialPortDataManager;
use utils::{args, variables::VERSION};

fn print_help() {
    println!("# Secure Vault Personal Information (SVPI)");
    println!("# Version: {}", VERSION);

    println!("{}", "=".repeat(107));
    println!("| {:50} | {:50} |", "Command", "Description");
    println!("{}", "=".repeat(107));
    println!("| {:50} | {:50} |", "svpi init / i <memory_size>", "Initialize the device for the desired memory architecture");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi format / f", "Format the data in the device");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi list / l", "Print all data list");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi set / s <name> <data>", "Set data");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi get / g <name>", "Get data");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi remove / r <name>", "Remove data");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi optimize / o", "Optimize the memory");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi version / v", "Print the version of the application");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi help / h", "Print this help message");
    println!("{}", "-".repeat(107));
    
    println!("{}", "=".repeat(107));
    println!("| {:50} | {:50} |", "Flags", "Description");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi <command> [--flags...] <data>", "How to use flags");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi list/set --password / -p ...", "Use password for encryption/decryption");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi set --password2 / -p2 ...", "Use password with confirmation for encryption");
    println!("{}", "-".repeat(107));
    println!("| {:50} | {:50} |", "svpi set/get --clipboard / -c", "Copy data to/from clipboard");
    println!("{}", "-".repeat(107));
}

fn main() -> std::io::Result<()> {
    match args::get_command() {
        Some(cmd) => {
            match cmd.as_str() {
                "init" | "i" => {
                    let memory_size = std::env::args().nth(2).expect("Memory size is required!").parse::<u32>().expect("Memory size must be a number!");
                    svpi::init_segments(memory_size)?;
                },
                "format" | "f" => {
                    svpi::format_data()?;
                },
                "list" | "l" => {
                    svpi::print_segments_info()?;
                },
                "set" | "s" => {
                    let name = args::get_param(0).expect("Name is required!");
                    let clipboard = args::get_flag(vec!["--clipboard", "-c"]);
                    let data = if clipboard.is_some() {
                        let mut clipboard = Clipboard::new().expect("Failed to create clipboard instance!");
                        clipboard.get_text().expect("Failed to get text from clipboard!")
                    } else {
                        args::get_param(1).expect("Data is required!")
                    };
                    svpi::set_segment(&name, &data)?;
                },
                "get" | "g" => {
                    let name = args::get_param(0).expect("Name is required!");
                    let data = svpi::get_segment(&name)?;
                    if let Some(data) = data {
                        let clipboard = args::get_flag(vec!["--clipboard", "-c"]);
                        if clipboard.is_some() {
                            let mut clipboard = Clipboard::new().expect("Failed to create clipboard instance!");
                            clipboard.set_text(data).expect("Failed to set text to clipboard!");
                            println!("Data copied to clipboard!");
                        }
                    }
                },
                "remove" | "r" => {
                    let name = args::get_param(0).expect("Name is required!");
                    svpi::remove_segment(&name)?;
                },
                "optimize" | "o" => {
                    svpi::optimize()?;
                },
                "version" | "v" => {
                    println!("Secure Vault Personal Information (SVPI) {}", VERSION);
                },
                "help" | "h" => {
                    print_help();
                },
                _ => {
                    println!("Invalid command!");
                    println!("Run `svpi help` to see the list of available commands.");
                }
            }
        },
        None => {
            print_help();
        }
    }

    Ok(())
}
