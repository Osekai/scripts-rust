use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    string::FromUtf8Error,
};

use bytes::Bytes;
use eyre::{Report, Result};

pub struct OsekaiResponse(String);

impl OsekaiResponse {
    pub fn new(bytes: Bytes) -> Result<Self> {
        Self::try_from(bytes)
    }
}

impl TryFrom<Bytes> for OsekaiResponse {
    type Error = Report;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        String::from_utf8(bytes.into())
            .map(Self)
            .map_err(FromUtf8Error::into_bytes)
            .map_err(|bytes| eyre!("Invalid UTF-8 response: {bytes:?}"))
    }
}

impl Display for OsekaiResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.0.is_empty() {
            Ok(())
        } else {
            write!(f, "; Response: {}", self.0)
        }
    }
}
