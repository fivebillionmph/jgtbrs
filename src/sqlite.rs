use rusqlite::{Connection, params, Row, Rows};
use anyhow::Result as Res;
use crate::error::AppError;

pub trait DBObject: Sized {
	fn table_name() -> &'static str;
	fn get_id(&self) -> i64;
	fn from_row(row: &Row<'_>) -> Res<Self>;

	fn load_by_primary_key(cxn: &Connection, id: i64) -> Res<Self> {
		let mut stmt = cxn.prepare(&format!(r#"
			SELECT *
			FROM {}
			WHERE id = ?
		"#, Self::table_name()))?;
		let mut rows = stmt.query(params![id])?;

		match Self::from_single_row(&mut rows)? {
			Some(x) => Ok(x),
			None => Err(AppError::new("Row with primary key does not exist").into()),
		}
	}

	fn from_single_row(rows: &mut Rows<'_>) -> Res<Option<Self>> {
		if let Some(row) = rows.next()? {
			Ok(Some(Self::from_row(row)?))
		} else {
			Ok(None)
		}
	}

	fn from_multiple_rows(rows: &mut Rows<'_>) -> Res<Vec<Self>> {
		let mut results = Vec::new();
		while let Some(row) = rows.next()? {
			results.push(Self::from_row(row)?);
		}
		Ok(results)
	}
}

pub struct Migrator {
	migrations: Vec<Migration>,
}

impl Migrator {
	pub fn new() -> Self {
		Self {
			migrations: Vec::new(),
		}
	}

	pub fn add_migration(&mut self, migration: Migration) -> Res<()> {
		if self.migrations.iter().find(|x| x.version == migration.version).is_some() {
			return Err(AppError::new("Two migrations with the same version are not allowed").into());
		}

		self.migrations.push(migration);

		Ok(())
	}

	pub fn run_migrations(&mut self, cxn: &Connection) -> Res<()> {
		self.migrations.sort_by(|a, b| a.version.cmp(&b.version));

		for migration in &self.migrations {
			migration.run(cxn)?;
		}

		Ok(())
	}
}

pub struct Migration {
	version: u32,
	statements: Vec<String>,
}

impl Migration {
	pub fn new(version: u32, statements: Vec<String>) -> Self {
		Self {
			version,
			statements,
		}
	}

	fn run(&self, cxn: &Connection) -> Res<()> {
		if !self.should_run_migration(cxn)? {
			return Ok(());
		}

		self.migrate(cxn)?;

		self.set_migration_version(cxn)?;
		Ok(())
	}

	fn should_run_migration(&self, cxn: &Connection) -> Res<bool> {
		let mut stmt = cxn.prepare("PRAGMA user_version")?;
		let mut rows = stmt.query(params![])?;
		if let Some(row) = rows.next()? {
			let db_version: u32 = row.get("user_version")?;
			Ok(db_version < self.version)
		} else {
			Ok(true)
		}
	}

	fn set_migration_version(&self, cxn: &Connection) -> Res<()> {
		let mut stmt = cxn.prepare(&format!("PRAGMA user_version = {}", self.version))?;
		stmt.execute(params![])?;
		Ok(())
	}

	fn migrate(&self, cxn: &Connection) -> Res<()> {
		for stmt in &self.statements {
			let mut stmt = cxn.prepare(&stmt)?;
			stmt.execute(params![])?;
		}
		Ok(())
	}
}