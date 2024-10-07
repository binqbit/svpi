use serialport::SerialPortType;

use crate::SerialPortDataManager;


impl SerialPortDataManager {
    pub fn find_device() -> SerialPortDataManager {
        let available_ports = serialport::available_ports().expect("Failed to get available ports");
        let ports: Vec<String> = available_ports
            .iter()
            .filter_map(|port| if let SerialPortType::UsbPort(_) = &port.port_type {
                Some(port)
            } else {
                None
            })
            .map(|port| port.port_name.clone())
            .collect();

        if ports.is_empty() {
            panic!("Device not found");
        }

        let path = if ports.len() > 1 {
            eprintln!("Multiple devices found:");
            for (i, port) in ports.iter().enumerate() {
                println!("{}: {}", i + 1, port);
            }

            println!("Select device:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Failed to read line");
            let index: usize = input.trim().parse().expect("Expected a number of device");
            if index == 0 || index > ports.len() {
                panic!("Device not found by number");
            }
            ports[index - 1].clone()
        } else {
            ports[0].clone()
        };

        SerialPortDataManager::new(&path).expect("Failed to connect to device")
    }
}