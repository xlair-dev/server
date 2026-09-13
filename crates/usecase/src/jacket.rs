use std::{future::Future, pin::Pin};

use thiserror::Error;

pub const MAX_JACKET_SIZE: usize = 5 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum JacketUploadError {
    #[error("unsupported jacket content type")]
    UnsupportedContentType,
    #[error("jacket image exceeds the 5 MiB limit")]
    TooLarge,
    #[error("uploaded object is not a valid image")]
    InvalidImage,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JacketUpload {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

impl JacketUpload {
    pub fn new(content_type: String, bytes: Vec<u8>) -> Result<Self, JacketUploadError> {
        if !matches!(
            content_type.as_str(),
            "image/jpeg" | "image/png" | "image/webp"
        ) {
            return Err(JacketUploadError::UnsupportedContentType);
        }
        if bytes.len() > MAX_JACKET_SIZE {
            return Err(JacketUploadError::TooLarge);
        }

        let valid = match content_type.as_str() {
            "image/jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
            "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            "image/webp" => bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
            _ => false,
        };
        if !valid {
            return Err(JacketUploadError::InvalidImage);
        }

        Ok(Self {
            content_type,
            bytes,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UploadedJacket {
    pub url: String,
}

pub trait JacketStorage: Send + Sync {
    fn upload<'a>(
        &'a self,
        music_id: &'a str,
        jacket: JacketUpload,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<UploadedJacket>> + Send + 'a>>;

    fn delete<'a>(
        &'a self,
        music_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;
}
