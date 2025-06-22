use crate::{spdm::DeviceError, SerialPortDataManager};

impl SerialPortDataManager {
    pub fn connect_to_device(is_console: bool) -> Result<SerialPortDataManager, DeviceError> {
        let ports: Vec<String> = SerialPortDataManager::get_available_ports()?;

        let path = if is_console && ports.len() > 1 {
            eprintln!("Multiple devices found:");
            for (i, port) in ports.iter().enumerate() {
                println!("{}: {}", i + 1, port);
            }

            println!("Select device:");
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let index: usize = input.trim().parse().expect("Expected a number of device");
            if index == 0 || index > ports.len() {
                panic!("Device not found by number");
            }
            ports[index - 1].clone()
        } else {
            ports[0].clone()
        };

        SerialPortDataManager::new(&path)
    }
}
