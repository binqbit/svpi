use seg_mgr::ARCHITECTURE_VERSION;

use crate::utils::args;

// mod api;
mod data_mgr;
mod pass_mgr;
mod seg_mgr;
mod svpi;
mod utils;

fn print_info() {
    println!("# Secure Vault Personal Information (SVPI)");
    println!("{}", "=".repeat(59));
    println!("| {:32} | {:20} |", "Info", "Value");
    println!("{}", "=".repeat(59));
    println!(
        "| {:32} | {:>20} |",
        "App Version",
        env!("CARGO_PKG_VERSION")
    );
    println!("{}", "-".repeat(59));
    println!(
        "| {:32} | {:20} |",
        "Architecture Version", ARCHITECTURE_VERSION
    );
    println!("{}", "-".repeat(59));
}

fn print_help() {
    print_info();

    println!();

    println!("{}", "=".repeat(107));
    println!("| {:50} | {:50} |", "Command", "Description");
    println!("{}", "=".repeat(107));

    for (cmd, desc) in svpi::HELP_COMMANDS {
        println!("| {:50} | {:50} |", cmd, desc);
        println!("{}", "-".repeat(107));
    }

    println!();

    println!("{}", "=".repeat(107));
    println!("| {:50} | {:50} |", "Flags", "Description");
    println!("{}", "-".repeat(107));

    for (flag, desc) in svpi::HELP_FLAGS {
        println!("| {:50} | {:50} |", flag, desc);
        println!("{}", "-".repeat(107));
    }
}

#[tokio::main]
async fn main() {
    match args::get_command() {
        Some(cmd) => match cmd.as_str() {
            "init" | "i" => {
                svpi::device::init_device();
            }
            "check" => {
                svpi::device::check_device();
            }
            "format" | "f" => {
                svpi::device::format_device();
            }
            "optimize" | "o" => {
                svpi::device::optimize_device();
            }
            "export" | "e" => {
                svpi::device::export_data();
            }
            "import" | "m" => {
                svpi::device::import_data();
            }
            "dump" | "d" => {
                svpi::device::save_dump();
            }
            "load" | "ld" => {
                svpi::device::load_dump();
            }

            "set-master-password" | "set-master" => {
                svpi::password::set_master_password();
            }
            "reset-master-password" | "reset-master" => {
                svpi::password::reset_master_password();
            }
            "check-master-password" | "check-master" => {
                svpi::password::check_master_password();
            }
            "add-encryption-key" | "add-key" => {
                svpi::password::add_encryption_key();
            }
            "link-key" | "link" => {
                svpi::password::link_key();
            }
            "sync-keys" | "sync" => {
                svpi::password::sync_keys();
            }

            "list" | "l" => {
                svpi::data::get_data_list();
            }
            "set" | "s" => {
                svpi::data::save_data();
            }
            "get" | "g" => {
                svpi::data::get_data();
            }
            "remove" | "r" => {
                svpi::data::remove_data();
            }
            "rename" | "rn" => {
                svpi::data::rename_data();
            }
            "change-data-type" | "cdt" => {
                svpi::data::change_data_type();
            }
            "change-password" | "cp" => {
                svpi::data::change_password();
            }

            "version" | "v" => {
                print_info();
            }
            "help" | "h" => {
                print_help();
            }
            "api-server" => {
                // api::server::api_server()
                //     .launch()
                //     .await
                //     .expect("Failed to start API server!");
            }
            "api-chrome" => {
                // api::chrome::run_chrome_app().expect("Failed to launch Chrome app!");
            }
            _ => {
                println!("Invalid command!");
                println!("Run `svpi help` to see the list of available commands.");
            }
        },
        None => {
            print_help();
        }
    }
}
