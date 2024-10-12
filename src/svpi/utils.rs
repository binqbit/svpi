use crate::{seg_mgmt::{Segment, SegmentManager}, spdm::SerialPortDataManager};


pub fn load_segments_info() -> std::io::Result<Option<SegmentManager>> {
    let spdm = SerialPortDataManager::find_device();
    let mut seg_mgmt = spdm.into_segment_manager();
    if seg_mgmt.load_segments()? {
        Ok(Some(seg_mgmt))
    } else {
        println!("Device not initialized!");
        println!("Please run the init command: svpi init <memory_size>");
        Ok(None)
    }
}

pub fn get_segment_manager() -> std::io::Result<SegmentManager> {
    let spdm = SerialPortDataManager::find_device();
    Ok(spdm.into_segment_manager())
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

pub fn print_segment(segment: &mut SegmentManager, seg: &Segment) -> std::io::Result<()> {
    let data = segment.read_segment_data(seg)?;
    let name = seg.get_name();
    let line_size = 9 + name.len() + data.len();
    println!("{}", "-".repeat(line_size));
    println!("| {} = {:?} |", name, data);
    println!("{}", "-".repeat(line_size));
    Ok(())
}
