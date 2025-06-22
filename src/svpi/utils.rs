use std::{fs::File, io::Write};

use super::result::Result;
use crate::{
    seg_mgmt::{Data, FormattedData, Segment, SegmentError, SegmentManager},
    spdm::SerialPortDataManager,
    svpi::result::Error,
    utils::{args, terminal},
};

pub fn get_inited_manager() -> Result<Option<SegmentManager>> {
    let mut seg_mgmt = SerialPortDataManager::connect_to_device(true)
        .map(|spdm| spdm.into_segment_manager())
        .map_err(Into::into)?;

    if !seg_mgmt.check_init_data().map_err(Into::into)? {
        println!("Device not initialized!");
        println!("Please run the init command: svpi init <memory_size>");
        return Ok(None);
    }

    if !seg_mgmt.check_architecture_version().map_err(Into::into)? {
        println!("Device architecture version mismatch!");
        println!("Please migrate the data from the old software version.");
        return Ok(None);
    }

    seg_mgmt.load_segments().map_err(Into::into)?;

    Ok(Some(seg_mgmt))
}

pub fn get_segment_manager() -> Result<SegmentManager> {
    SerialPortDataManager::connect_to_device(true)
        .map(|spdm| spdm.into_segment_manager())
        .map_err(Into::into)
}

pub fn get_password(
    seg_mgmt: &mut SegmentManager,
    check_flag: bool,
    confirm: bool,
) -> Result<Option<String>> {
    let password = terminal::get_password(check_flag, confirm, None);

    if let Some(password) = &password {
        if args::check_flag(vec!["--root-password", "-rp"]) {
            return seg_mgmt.get_root_password(&password, None).map(Some);
        }
    }

    Ok(password)
}

pub fn print_memory_state(seg_mgmt: &SegmentManager, optimized_size: Option<u32>) {
    println!("{}", "=".repeat(36));
    println!("| {:14} | {:15} |", "Memory Size", "Value (bytes)");
    println!("{}", "=".repeat(36));
    println!("| {:14} | {:15} |", "Total", seg_mgmt.memory_size);
    println!("{}", "-".repeat(36));
    println!("| {:14} | {:15} |", "Free", seg_mgmt.free_memory_size());
    println!("{}", "-".repeat(36));
    if let Some(optimized_size) = optimized_size {
        println!("| {:14} | {:15} |", "Optimized", optimized_size);
        println!("{}", "-".repeat(36));
    }
}

pub fn print_segments(
    mut segs: Vec<&mut Segment>,
    password: Option<&str>,
    is_view: bool,
) -> Result<()> {
    let is_view = (is_view && !args::check_flag(vec!["--clipboard", "-c"]))
        || args::check_flag(vec!["--view", "-v"]);

    println!("{}", "=".repeat(112));
    println!("| {:32} | {:20} | {:50} |", "Name", "Data Type", "Data");
    println!("{}", "=".repeat(112));

    for seg in segs.iter_mut() {
        let data = seg.read_data(password).map_err(Into::into)?;
        let is_encrypted = if let Data::Encrypted(_) = data {
            true
        } else {
            false
        };
        let data = data.to_string().map_err(Into::into)?;
        let data = if is_view {
            data
        } else {
            format!("{} bytes", seg.size)
        };
        let data_type = if is_encrypted {
            format!("@{:?}", seg.data_type)
        } else {
            format!("{:?}", seg.data_type)
        };

        println!(
            "| {:32} | {:20?} | {:50} |",
            seg.get_name(),
            data_type,
            data
        );
        println!("{}", "-".repeat(112));
    }
    Ok(())
}

pub fn export_to_file(
    seg_mgmt: &mut SegmentManager,
    file_path: &str,
    password: Option<&str>,
) -> Result<()> {
    let mut list = Vec::new();

    let mut segs = seg_mgmt.get_active_segments_mut();
    for seg in segs.iter_mut() {
        let data = seg.to_formatted_data(password).map_err(Into::into)?;
        let formatted_data = data.encode().map_err(Into::into)?;
        list.push(formatted_data);
    }

    let str = list.join("\n");
    let mut file = File::create(file_path).map_err(Error::IoError)?;
    file.write_all(str.as_bytes()).map_err(Error::IoError)?;
    Ok(())
}

pub fn import_from_file(
    seg_mgmt: &mut SegmentManager,
    file_path: &str,
    password: Option<&str>,
) -> Result<()> {
    let str = std::fs::read_to_string(file_path).map_err(Error::IoError)?;
    let list = str.lines();

    for str in list {
        let formatted_data = FormattedData::decode(str, password).map_err(Into::into)?;
        let password = if let Data::Encrypted(_) = &formatted_data.data {
            None
        } else {
            password
        };

        let seg = seg_mgmt
            .set_segment(&formatted_data.name, formatted_data.data, password)
            .map_err(Into::into)?;
        if seg.is_none() {
            println!("Not enough memory!");
            println!("Please optimize the memory: svpi optimize");
            break;
        }
        let seg = seg_mgmt
            .find_segment_by_name(&formatted_data.name)
            .ok_or(Error::SegmentError(SegmentError::NotFound))?;
        seg.set_type(formatted_data.data_type).map_err(Into::into)?;
    }
    Ok(())
}

pub fn save_dump_to_file(seg_mgmt: &mut SegmentManager, file_path: &str) -> Result<()> {
    let data = seg_mgmt.get_dump().map_err(Into::into)?;
    let mut file = File::create(file_path).map_err(Error::IoError)?;
    file.write_all(&data).map_err(Error::IoError)?;
    Ok(())
}

pub fn load_dump_from_file(seg_mgmt: &mut SegmentManager, file_path: &str) -> Result<()> {
    let data = std::fs::read(file_path).map_err(Error::IoError)?;
    seg_mgmt.set_dump(&data).map_err(Into::into)?;
    Ok(())
}
