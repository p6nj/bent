use std::{ffi::OsString, ops::Deref, path::Path};

use image::ImageFormat;
use rfd::FileHandle;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct ImageFile(String);

#[derive(Error, Debug)]
pub(super) enum ImageFileError {
    #[error("{0:?} is not a known image file extension")]
    UnknownExtension(OsString),
    #[error("can't decode an image without a file extension")]
    NoExtension,
}

pub(super) type ImageFileResult<T> = Result<T, ImageFileError>;

impl ImageFile {
    pub(super) fn try_new(file: &FileHandle) -> ImageFileResult<Self> {
        let binding = file.file_name();
        let path = Path::new(&binding);
        let extension = path.extension().ok_or(ImageFileError::NoExtension)?;
        if ImageFormat::from_path(path).is_err() {
            return Err(ImageFileError::UnknownExtension(extension.to_os_string()));
        }
        Ok(Self(file.file_name()))
    }
}

impl Deref for ImageFile {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
