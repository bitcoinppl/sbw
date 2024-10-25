use std::{io::Read as _, path::PathBuf};

use crate::multi_format::{MultiFormat, MultiFormatError};

#[derive(Debug, Clone, uniffi::Object)]
pub struct FileHandler {
    file_path: PathBuf,
}

#[derive(Debug, Clone, uniffi::Error, thiserror::Error, derive_more::Display)]
pub enum FileHandlerError {
    /// File not found
    FileNotFound,

    /// Unable to open file {0}
    OpenFile(String),

    /// Unable to to read file {0}
    ReadFile(String),

    /// File is not a recognized format: {0:?}
    NotRecognizedFormat(#[from] MultiFormatError),
}

#[uniffi::export]
impl FileHandler {
    #[uniffi::constructor]
    pub fn new(file_path: String) -> Self {
        Self {
            file_path: PathBuf::from(file_path),
        }
    }

    #[uniffi::method]
    pub fn read(&self) -> Result<MultiFormat, FileHandlerError> {
        if !self.file_path.exists() {
            return Err(FileHandlerError::FileNotFound);
        }

        let file = std::fs::File::open(&self.file_path)
            .map_err(|e| FileHandlerError::OpenFile(e.to_string()))?;

        let data = file
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|e| FileHandlerError::ReadFile(e.to_string()))?;

        let string_or_data = crate::multi_format::StringOrData::new(data);

        let multi_format = string_or_data.try_into()?;
        Ok(multi_format)
    }
}
