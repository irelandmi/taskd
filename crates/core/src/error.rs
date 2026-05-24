use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("not found: {0}")]
	NotFound(String),

	#[error("conflict: {0}")]
	Conflict(String),

	#[error("invalid input: {0}")]
	InvalidInput(String),

	#[error("database error: {0}")]
	Database(#[from] rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
