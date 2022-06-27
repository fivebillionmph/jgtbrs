use std::path::Path;
use anyhow::Result as Res;
use crate::error::AppError;

pub fn get_unix_timestamp() -> Res<u64> {
	use std::time::SystemTime;
	Ok(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs())
}

pub fn path_to_string(path: &Path) -> Res<String> {
	Ok(path.as_os_str().to_str().ok_or(AppError::new("Invalid path name"))?.to_string())
}
