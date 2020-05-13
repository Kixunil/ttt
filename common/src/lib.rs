//! Common functions used by different components of Trivial Time Tracker
//!
//! The main point of this library is for all components to correctly access same paths and init
//! the database correctly.

use std::path::{Path, PathBuf};

pub use rusqlite;

/// Error type returned when data dir can't be initialized.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DataDirError(InnerDataDirError);

#[derive(Debug, thiserror::Error)]
enum InnerDataDirError {
    #[error("unknown data (home) directory")]
    UnknownHome,
    #[error("failed to create data (home) directory {path}")]
    CreateFailed { path: PathBuf, #[source] error: std::io::Error },
}

/// Prepares default data dir and returns it.
///
/// Fails if the data dir can't be found or can't be created.
pub fn default_data_dir() -> Result<PathBuf, DataDirError> {
    let mut data_dir = dirs::data_dir()
        .ok_or(DataDirError(InnerDataDirError::UnknownHome))?;

    data_dir.push("ttt");

    if let Err(error) = std::fs::create_dir_all(&data_dir) {
        return Err(DataDirError(InnerDataDirError::CreateFailed {
            path: data_dir,
            error,
        }))
    }

    Ok(data_dir)
}

/// Connects to the database and initializes it.
///
/// `data_dir` should be the value returned from `default_data_dir()`, it's split so that failures
/// to connect to the database can be logged.
pub fn db_connect<P: AsRef<Path>>(data_dir: P) -> rusqlite::Result<rusqlite::Connection> {
    let conn = rusqlite::Connection::open(data_dir.as_ref().join("records.db"))?;

    conn.execute_batch(include_str!("../init_tables.sql"))?;

    Ok(conn)
}
