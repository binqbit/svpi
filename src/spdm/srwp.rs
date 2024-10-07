use crate::spdm::SerialPortDataManager;

const SRWP_CMD: u8 = 0x00;
const CMD_TEST: u8 = 0x00;
const CMD_READ: u8 = 0x01;
const CMD_WRITE: u8 = 0x02;

impl SerialPortDataManager {
    pub fn test(&mut self, data: &[u8]) -> std::io::Result<Vec<u8>> {
        self.clear()?;

        let mut buffer = vec![0u8; 6 + data.len()];
        buffer[0] = SRWP_CMD;
        buffer[1] = CMD_TEST;
        buffer[2..6].copy_from_slice(&(data.len() as u32).to_le_bytes());
        buffer[6..].copy_from_slice(data);

        self.write_data_terminal_ready(true)?;
        self.write_request_to_send(true)?;
        self.write(&buffer)?;
        self.flush()?;
        self.write_request_to_send(false)?;
        self.write_data_terminal_ready(false)?;

        let mut data = vec![0u8; data.len()];
        self.read(&mut data)?;
        Ok(data)
    }

    pub fn read_data(&mut self, address: u32, length: u32) -> std::io::Result<Vec<u8>> {
        self.clear()?;

        let mut buffer = vec![0u8; 10];
        buffer[0] = SRWP_CMD;
        buffer[1] = CMD_READ;
        buffer[2..6].copy_from_slice(&address.to_le_bytes());
        buffer[6..10].copy_from_slice(&length.to_le_bytes());

        self.write_data_terminal_ready(true)?;
        self.write_request_to_send(true)?;
        self.write(&buffer)?;
        self.flush()?;
        self.write_request_to_send(false)?;
        self.write_data_terminal_ready(false)?;

        self.read_data_set_ready()?;
        let mut data = vec![0u8; length as usize];
        self.read(&mut data)?;
        Ok(data)
    }

    pub fn write_data(&mut self, address: u32, data: &[u8]) -> std::io::Result<()> {
        self.clear()?;

        let mut buffer = vec![0u8; 10 + data.len()];
        buffer[0] = SRWP_CMD;
        buffer[1] = CMD_WRITE;
        buffer[2..6].copy_from_slice(&address.to_le_bytes());
        buffer[6..10].copy_from_slice(&(data.len() as u32).to_le_bytes());
        buffer[10..].copy_from_slice(data);

        self.write_data_terminal_ready(true)?;
        self.write_request_to_send(true)?;
        self.write(&buffer)?;
        self.flush()?;
        self.write_request_to_send(false)?;
        self.write_data_terminal_ready(false)?;
        Ok(())
    }
}