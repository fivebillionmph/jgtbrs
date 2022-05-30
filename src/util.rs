use anyhow::Result as Res;

pub fn get_unix_timestamp() -> Res<u64> {
	use std::time::SystemTime;
	Ok(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs())
}
