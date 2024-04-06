use std::path::{Path, PathBuf};

use tempfile::{NamedTempFile, Builder};

pub fn named_tempfile_in<P: AsRef<Path>>(dir: P) -> std::io::Result<NamedTempFile> {
    Builder::new().tempfile_in(dir)
}


#[derive(Debug, thiserror::Error)]
pub enum TempfileError {
    #[error("path is missing parent directory: {0}")]
    MissingDirectory(Box<PathBuf>),
    #[error("path is missing filename: {0}")]
    MissingFilename(Box<PathBuf>),
    #[error("could not create tempfile: {0}")]
    IOError(#[from] Box<std::io::Error>)
}

pub fn named_tempfile_for<P: AsRef<Path>>(targetfile: P) -> Result<NamedTempFile, TempfileError> {
    let targetfilepath = targetfile.as_ref();
    let dirpath = targetfilepath.parent().ok_or_else(
        || TempfileError::MissingDirectory(targetfilepath.to_owned().into()))?
        .to_owned();
    let filename = targetfilepath.file_name().ok_or_else(
        || TempfileError::MissingFilename(targetfilepath.to_owned().into()))?;
    Ok(Builder::new()
       .prefix(&format!("{}.", filename.to_string_lossy()))
       .suffix(".tmp")
       .tempfile_in(dirpath).map_err(Box::new)?)
}


// now for the more complicated bits?

