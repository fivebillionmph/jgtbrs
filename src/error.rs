use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct AppError {
	message: String,
}
impl AppError {
	pub fn new(message: &str) -> Self {
		Self {
			message: String::from(message),
		}
	}
}
impl Error for AppError {}
impl fmt::Display for AppError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.message)
	}
}
