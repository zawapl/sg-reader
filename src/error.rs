use std::io::Error;
use std::str::Utf8Error;

pub type Result<T> = std::result::Result<T, SgImageError>;

#[derive(Debug)]
pub enum SgImageError {
    InvalidHeader,
    ImageDataLengthMismatch,
    UnknownImageType(u16),
    IoError(Error),
    Utf8Error(Utf8Error),
}

impl From<Error> for SgImageError {
    fn from(value: Error) -> Self {
        return SgImageError::IoError(value);
    }
}
