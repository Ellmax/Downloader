use std::path::PathBuf;

pub struct Part {
    pub start: u64,
    pub end: Option<u64>,
    pub temp_path: PathBuf,
}
