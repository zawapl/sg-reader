use std::fmt::{Display, Formatter};
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

impl Display for SgImageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SgImageError::InvalidHeader => write!(f, "invalid header enountered"),
            SgImageError::ImageDataLengthMismatch => write!(f, "data length mismatch detected"),
            SgImageError::UnknownImageType(_) => write!(f, "unknown image type encountered"),
            SgImageError::IoError(err) => write!(f, "IO error enountered: {}", err),
            SgImageError::Utf8Error(_) => write!(f, "error enountered when reading UTF8 srting"),
        }
        
    }
}

impl From<Error> for SgImageError {
    fn from(value: Error) -> Self {
        SgImageError::IoError(value)
    }
}

impl std::error::Error for SgImageError {}
