use std::io::{self, Read, Write};
use commands::{get_data, list, status, CommandRequest, Status};
use serde_json::{from_str, json, to_string, Value};

use crate::seg_mgmt::SegmentManager;
use super::seg_mgmt::DeviceStatus;

mod commands;

fn read_message() -> io::Result<CommandRequest> {
    let mut length_buf = [0u8; 4];
    io::stdin().read_exact(&mut length_buf)?;
    let length = u32::from_ne_bytes(length_buf) as usize;

    let mut message_buf = vec![0u8; length];
    io::stdin().read_exact(&mut message_buf)?;

    let command: CommandRequest = from_str(&String::from_utf8(message_buf).unwrap()).unwrap();
    Ok(command)
}

fn send_message(response: &Value) -> io::Result<()> {
    let serialized = to_string(response).unwrap();
    let length = serialized.len() as u32;
    io::stdout().write_all(&length.to_ne_bytes())?;
    io::stdout().write_all(serialized.as_bytes())?;
    io::stdout().flush()?;
    Ok(())
}

fn process_command(command: CommandRequest, seg_mgmt: &mut SegmentManager) -> Result<Value, Status> {
    Ok(match command {
        CommandRequest::Status {} => serde_json::to_value(status::status(seg_mgmt)?).unwrap(),
        CommandRequest::List {} => serde_json::to_value(list::list(seg_mgmt)?).unwrap(),
        CommandRequest::GetData(req) => serde_json::to_value(get_data::get_data(req, seg_mgmt)?).unwrap(),
    })
}

pub fn run_chrome_app() -> io::Result<()> {
    let request = read_message()?;

    let response = match DeviceStatus::connect_device() {
        DeviceStatus::Some(mut seg_mgmt) => {
            match process_command(request, &mut seg_mgmt) {
                Ok(response) => response,
                Err(status) => json!({ "status": status }),
            }
        },
        DeviceStatus::DeviceNotFound => json!({ "status": Status::DeviceNotFound }),
        DeviceStatus::DeviceError => json!({ "status": Status::DeviceError }),
    };

    send_message(&response)?;

    Ok(())
}
