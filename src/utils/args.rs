pub fn get_command() -> Option<String> {
    std::env::args().nth(1)
}

fn skip_flags() -> usize {
    let mut skip = 2;
    for arg in std::env::args().skip(2) {
        if arg.starts_with("--") || arg.starts_with("-") {
            skip += 1;
        } else {
            break;
        }
    }
    skip
}

pub fn get_flag(flags: Vec<&str>) -> Option<String> {
    std::env::args()
        .skip(2)
        .find(|arg| flags.contains(&arg.as_str()))
}

pub fn get_param(id: usize) -> Option<String> {
    std::env::args().nth(skip_flags() + id)
}
