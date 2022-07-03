use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::thread;
use std::sync::Arc;
use tera::{Tera, Context, Error as TerraError, Result as TeraResult};
use parking_lot::RwLock;
use notify::{Watcher, watcher, RecursiveMode};
use anyhow::Result as Res;
use serde_json::Value;

pub struct Templater {
	templates: RwLock<Tera>,
}

impl Templater {
	pub fn new(dir: &str) -> Res<Arc<Self>> {
		let glob = dir.to_owned() + "/**/*.html";
		let mut templates = Tera::new(&glob)?;
		templates.register_filter("ts_datetime", ts_datetime);

		let templates = RwLock::new(templates);

		let manager = Arc::new(Self {
			templates,
		});

		let (tx, rx) = channel();

		let mut watcher = watcher(tx, Duration::from_secs(1))?;
		watcher.watch(dir, RecursiveMode::Recursive)?;

		let manager2 = manager.clone();
		thread::spawn(move || {
			let _ = watcher;
			loop {
				match rx.recv() {
					Ok(_) => {
						let mut templates = manager2.templates.write();
						let _ = templates.full_reload();
					}
					Err(_) => {}
				}
			}
		});

		Ok(manager)
	}

	pub fn render(&self, template: &str, context: &Context) -> tera::Result<String> {
		self.templates.read().render(template, context)
	}
}

fn ts_datetime(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
	use chrono::prelude::*;

	let timestamp = match value.as_i64() {
		Some(v) => v,
		None => {
			return Err(TerraError::msg("Invalid timestamp"));
		}
	};

	let naive = NaiveDateTime::from_timestamp(timestamp, 0);
	let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

	let format_str = if args.contains_key("date") {
		"%Y-%m-%d"
	} else {
		"%Y-%m-%d %H:%M:%S"
	};

	Ok(serde_json::json!(datetime.with_timezone(&Local).format(format_str).to_string()))
}
