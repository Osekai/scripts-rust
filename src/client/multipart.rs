use std::{fmt::Display, io::Write};

use eyre::{Context as _, Result};
use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;

const BOUNDARY_LEN: usize = 16;

/// Multipart form implementation to send through POST requests
pub struct Multipart {
    bytes: Vec<u8>,
    boundary: Box<str>,
}

impl Multipart {
    pub fn new() -> Self {
        let boundary = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .take(BOUNDARY_LEN)
            .map(|c| c as char)
            .collect::<String>()
            .into_boxed_str();

        Self {
            bytes: Vec::with_capacity(16_384),
            boundary,
        }
    }

    pub fn push_text<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Display,
        V: Display,
    {
        self.write_field_headers(key, None);
        let _ = write!(self.bytes, "{value}");

        self
    }

    pub fn push_json<K, J>(mut self, key: K, data: &J) -> Result<Self>
    where
        K: Display,
        J: Serialize,
    {
        self.write_field_headers(key, Some("application/json"));
        serde_json::to_writer(&mut self.bytes, data).context("failed to serialize data")?;

        Ok(self)
    }

    pub fn finish(mut self) -> Vec<u8> {
        if !self.is_empty() {
            self.bytes.extend_from_slice(b"\r\n");
        }

        let _ = write!(self.bytes, "--{}--\r\n", self.boundary);

        self.bytes
    }

    pub fn boundary(&self) -> &str {
        &self.boundary
    }

    fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    fn write_field_headers(&mut self, name: impl Display, content_type: Option<&str>) {
        if !self.is_empty() {
            self.bytes.extend_from_slice(b"\r\n");
        }

        let _ = write!(self.bytes, "--{}\r\n", self.boundary);

        let _ = write!(
            self.bytes,
            "Content-Disposition: form-data; name=\"{name}\""
        );

        if let Some(content_type) = content_type {
            let _ = write!(self.bytes, "\r\nContent-Type: {content_type}");
        }

        self.bytes.extend_from_slice(b"\r\n\r\n");
    }
}
