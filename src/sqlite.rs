use rusqlite::{Connection, params, Row, Rows};
use anyhow::Result;
use crate::error::AppError;

pub trait DBObject: Sized {
	fn table_name() -> &'static str;
	fn get_id(&self) -> i64;
	fn from_row(row: &Row<'_>) -> Result<Self>;

	fn load_by_primary_key(cxn: &Connection, id: i64) -> Result<Self> {
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

	fn from_single_row(rows: &mut Rows<'_>) -> Result<Option<Self>> {
		if let Some(row) = rows.next()? {
			Ok(Some(Self::from_row(row)?))
		} else {
			Ok(None)
		}
	}

	fn from_multiple_rows(rows: &mut Rows<'_>) -> Result<Vec<Self>> {
		let mut results = Vec::new();
		while let Some(row) = rows.next()? {
			results.push(Self::from_row(row)?);
		}
		Ok(results)
	}
}
