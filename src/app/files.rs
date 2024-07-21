use std::{ffi::OsStr, ops::Deref, path::PathBuf};

use image::ImageFormat;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct ImageFile(PathBuf);

#[derive(Error, Debug)]
pub(super) enum ImageFileError<'a> {
    #[error("{0:?} is not a known image file extension")]
    UnknownExtension(&'a OsStr),
    #[error("can't decode an image without a file extension")]
    NoExtension,
}

pub(super) type ImageFileResult<'a, T> = Result<T, ImageFileError<'a>>;

impl ImageFile {
    pub(super) fn try_new<'a>(path: &'a PathBuf) -> ImageFileResult<'a, Self> {
        let extension = path.extension().ok_or(ImageFileError::NoExtension)?;
        if ImageFormat::from_path(&path).is_err() {
            return Err(ImageFileError::UnknownExtension(extension));
        }
        Ok(Self(path.to_path_buf()))
    }
}

impl Deref for ImageFile {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
