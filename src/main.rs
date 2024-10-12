
mod spdm;
mod seg_mgmt;
mod svpi;

use std::io::Write;

use spdm::SerialPortDataManager;

fn print_help() {
    println!("# Secure Vault Personal Information (SVPI)");
    println!("# Version: v1.0");
    println!("{}", "=".repeat(107));
    println!("| {:40} | {:60} |", "Command", "Description");
    println!("{}", "=".repeat(107));
    println!("| {:40} | {:60} |", "svpi init / i <memory_size>", "Initialize the device for the desired memory architecture");
    println!("{}", "-".repeat(107));
    println!("| {:40} | {:60} |", "svpi format / f", "Format the data in the device");
    println!("{}", "-".repeat(107));
    println!("| {:40} | {:60} |", "svpi list / l", "Print all data list");
    println!("{}", "-".repeat(107));
    println!("| {:40} | {:60} |", "svpi set / s <name> <data>", "Set data");
    println!("{}", "-".repeat(107));
    println!("| {:40} | {:60} |", "svpi get / g <name>", "Get data");
    println!("{}", "-".repeat(107));
    println!("| {:40} | {:60} |", "svpi remove / r <name>", "Remove data");
    println!("{}", "-".repeat(107));
    println!("| {:40} | {:60} |", "svpi optimize / o", "Optimize the memory");
    println!("{}", "-".repeat(107));
    println!("| {:40} | {:60} |", "svpi version / v", "Print the version of the application");
    println!("{}", "-".repeat(107));
    println!("| {:40} | {:60} |", "svpi help / h", "Print this help message");
    println!("{}", "-".repeat(107));
}

fn main() -> std::io::Result<()> {
    match std::env::args().nth(1) {
        Some(cmd) => {
            match cmd.as_str() {
                "init" | "i" => {
                    print!("Are you sure you want to initialize the device? (y/N): ");
                    std::io::stdout().flush()?;
                    let mut confirm = String::new();
                    std::io::stdin().read_line(&mut confirm)?;
                    if confirm.trim() != "y" {
                        return Ok(());
                    }
                    let memory_size = std::env::args().nth(2).expect("Memory size is required!").parse::<u32>().expect("Memory size must be a number!");
                    svpi::init_segments(memory_size)?;
                },
                "format" | "f" => {
                    print!("Are you sure you want to format the data? (y/N): ");
                    std::io::stdout().flush()?;
                    let mut confirm = String::new();
                    std::io::stdin().read_line(&mut confirm)?;
                    if confirm.trim() != "y" {
                        return Ok(());
                    }
                    svpi::format_data()?;
                },
                "list" | "l" => {
                    svpi::print_segments_info()?;
                },
                "set" | "s" => {
                    let name = std::env::args().nth(2).expect("Name is required!");
                    let data = std::env::args().nth(3).expect("Data is required!");
                    svpi::set_segment(&name, &data)?;
                },
                "get" | "g" => {
                    let name = std::env::args().nth(2).expect("Name is required!");
                    svpi::get_segment(&name)?;
                },
                "remove" | "r" => {
                    let name = std::env::args().nth(2).expect("Name is required!");
                    svpi::remove_segment(&name)?;
                },
                "optimize" | "o" => {
                    svpi::optimize()?;
                },
                "version" | "v" => {
                    println!("Secure Vault Personal Information (SVPI) v1.0");
                },
                "help" | "h" => {
                    print_help();
                },
                _ => {
                    println!("Invalid command!");
                    println!("Run `svpi` to see the list of available commands.");
                }
            }
        },
        None => {
            print_help();
        }
    }

    Ok(())
}
