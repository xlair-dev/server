use std::{future::Future, io::Cursor, pin::Pin};

use image::{ImageFormat, ImageReader, Limits};
use thiserror::Error;

pub const MAX_JACKET_SIZE: usize = 5 * 1024 * 1024;
const MAX_JACKET_DIMENSION: u32 = 4096;
const MAX_JACKET_ALLOCATION: u64 = 64 * 1024 * 1024;
const JACKET_CONTENT_TYPE: &str = "image/png";

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

        let mut reader = ImageReader::new(Cursor::new(bytes));
        let mut limits = Limits::default();
        limits.max_image_width = Some(MAX_JACKET_DIMENSION);
        limits.max_image_height = Some(MAX_JACKET_DIMENSION);
        limits.max_alloc = Some(MAX_JACKET_ALLOCATION);
        reader.limits(limits);
        let image = reader
            .with_guessed_format()
            .map_err(|_| JacketUploadError::InvalidImage)?
            .decode()
            .map_err(|_| JacketUploadError::InvalidImage)?;
        let mut normalized = Cursor::new(Vec::new());
        image
            .write_to(&mut normalized, ImageFormat::Png)
            .map_err(|_| JacketUploadError::InvalidImage)?;
        if normalized.get_ref().len() > MAX_JACKET_SIZE {
            return Err(JacketUploadError::TooLarge);
        }

        Ok(Self {
            content_type: JACKET_CONTENT_TYPE.to_owned(),
            bytes: normalized.into_inner(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_supported_images_to_png() {
        let mut source = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(1, 1)
            .write_to(&mut source, ImageFormat::Jpeg)
            .unwrap();

        let jacket = JacketUpload::new("image/jpeg".to_owned(), source.into_inner()).unwrap();

        assert_eq!(jacket.content_type, "image/png");
        assert!(jacket.bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn rejects_invalid_image_data() {
        let error =
            JacketUpload::new("image/png".to_owned(), b"not an image".to_vec()).unwrap_err();

        assert!(matches!(error, JacketUploadError::InvalidImage));
    }
}
