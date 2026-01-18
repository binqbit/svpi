use std::io::{self, Read, Write};

use crate::{
    data_mgr::DataInterfaceType,
    protocol::api::{self, ApiTransport, CommandRequest},
    utils::response::SvpiResponse,
};

fn read_message() -> io::Result<CommandRequest> {
    let mut length_buf = [0u8; 4];
    io::stdin().read_exact(&mut length_buf)?;
    let length = u32::from_ne_bytes(length_buf) as usize;

    let mut message_buf = vec![0u8; length];
    io::stdin().read_exact(&mut message_buf)?;

    serde_json::from_slice(&message_buf)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn send_message(response: &SvpiResponse) -> io::Result<()> {
    let serialized = serde_json::to_vec(response)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let length = serialized.len() as u32;
    io::stdout().write_all(&length.to_ne_bytes())?;
    io::stdout().write_all(&serialized)?;
    io::stdout().flush()?;
    Ok(())
}

pub fn run_chrome_app(interface_type: DataInterfaceType) -> io::Result<()> {
    let request = read_message()?;
    let response = api::handle(ApiTransport::Chrome, interface_type, request);

    send_message(&response)?;

    Ok(())
}
