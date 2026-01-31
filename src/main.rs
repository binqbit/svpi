use clap::{error::ErrorKind, Parser};

use crate::{cli::Mode, utils::response::SvpiResponse};

mod api;
mod cli;
mod data_mgr;
mod pass_mgr;
mod protocol;
mod seg_mgr;
mod svpi;
mod utils;

#[tokio::main]
async fn main() {
    let mode_hint = std::env::args()
        .skip(1)
        .find_map(|arg| arg.strip_prefix("--mode=").map(|v| v.to_string()))
        .unwrap_or_default();
    let prefer_json_errors = mode_hint.trim().eq_ignore_ascii_case("json");

    let cli = match crate::cli::CliArgs::try_parse() {
        Ok(v) => v,
        Err(err) => {
            match err.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    print!("{err}");
                    std::process::exit(err.exit_code());
                }
                _ => {}
            }
            if prefer_json_errors {
                let resp = SvpiResponse::invalid_argument(None, "args", err.to_string());
                resp.print_json();
            } else {
                eprintln!("{err}");
            }
            std::process::exit(2);
        }
    };

    match cli.mode {
        Mode::Cli | Mode::Json => std::process::exit(svpi::cli_mode::run_with_cli(&cli)),
        Mode::Server => {
            if cli.command.is_some() {
                eprintln!("invalid_argument: subcommand is not supported in --mode=server");
                std::process::exit(2);
            }
            api::server::api_server(
                cli.interface_type(),
                cli.auto_exit,
                cli.bind,
                cli.port,
                cli.cors,
            )
            .launch()
            .await
            .expect("Failed to start API server!");
        }
        Mode::Chrome => {
            if cli.command.is_some() {
                eprintln!("invalid_argument: subcommand is not supported in --mode=chrome");
                std::process::exit(2);
            }
            api::chrome::run_chrome_app(cli.interface_type())
                .expect("Failed to run Chrome native app!");
        }
    }
}
