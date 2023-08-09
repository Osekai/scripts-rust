use eyre::{Context as _, Result};
use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;

const BOUNDARY_LEN: usize = 16;

/// Multipart form implementation to send through POST requests
pub struct Multipart {
    bytes: Vec<u8>,
    boundary: [u8; BOUNDARY_LEN],
}

impl Multipart {
    const BOUNDARY_TERMINATOR: &[u8] = b"--";
    const NEWLINE: &[u8] = b"\r\n";

    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut boundary = [0; BOUNDARY_LEN];

        boundary
            .iter_mut()
            .for_each(|byte| *byte = rng.sample(Alphanumeric));

        Self {
            bytes: Vec::with_capacity(16_384),
            boundary,
        }
    }

    pub fn push_text<K, V>(mut self, key: K, value: V) -> Self
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        self.write_field_headers(key, None);
        self.bytes.extend_from_slice(value.as_ref());

        self
    }

    pub fn push_json<K, J>(mut self, key: K, data: &J) -> Result<Self>
    where
        K: AsRef<[u8]>,
        J: Serialize,
    {
        self.write_field_headers(key, Some("application/json"));
        serde_json::to_writer(&mut self.bytes, data).context("failed to serialize data")?;

        Ok(self)
    }

    pub fn finish(mut self) -> Vec<u8> {
        if !self.bytes.is_empty() {
            self.bytes.extend_from_slice(Self::NEWLINE);
        }

        self.bytes.extend_from_slice(Self::BOUNDARY_TERMINATOR);
        self.bytes.extend_from_slice(&self.boundary);
        self.bytes.extend_from_slice(Self::BOUNDARY_TERMINATOR);

        self.bytes
    }

    pub fn content_type(&self) -> String {
        const PREFIX: &[u8] = b"multipart/form-data; boundary=";

        let mut content_type = Vec::with_capacity(PREFIX.len() + self.boundary.len());
        content_type.extend_from_slice(PREFIX);
        content_type.extend_from_slice(&self.boundary);

        // SAFETY: boundary only contains alphanumeric characters and thus is UTF-8 conform
        unsafe { String::from_utf8_unchecked(content_type) }
    }

    fn write_field_headers(&mut self, name: impl AsRef<[u8]>, content_type: Option<&str>) {
        if !self.bytes.is_empty() {
            self.bytes.extend_from_slice(Self::NEWLINE);
        }

        self.bytes.extend_from_slice(Self::BOUNDARY_TERMINATOR);
        self.bytes.extend_from_slice(&self.boundary);
        self.bytes.extend_from_slice(Self::NEWLINE);

        self.bytes
            .extend_from_slice(b"Content-Disposition: form-data; name=\"");
        self.bytes.extend_from_slice(name.as_ref());
        self.bytes.push(b'"');

        if let Some(content_type) = content_type {
            self.bytes.extend_from_slice(b"\r\nContent-Type: ");
            self.bytes.extend_from_slice(content_type.as_bytes());
        }

        self.bytes.extend_from_slice(Self::NEWLINE);
        self.bytes.extend_from_slice(Self::NEWLINE);
    }
}
