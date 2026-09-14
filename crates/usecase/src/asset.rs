use std::{future::Future, io::Cursor, pin::Pin};

use image::{ImageFormat, ImageReader, Limits};
use thiserror::Error;
use tokio::io::AsyncRead;

pub const MAX_JACKET_SIZE: usize = 5 * 1024 * 1024;
pub const MAX_AUDIO_SIZE: usize = 30 * 1024 * 1024;
pub const MAX_CHART_SIZE: usize = 5 * 1024 * 1024;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AssetKind {
    Jacket,
    Audio,
    Chart,
}

impl AssetKind {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Jacket => "jackets",
            Self::Audio => "musics",
            Self::Chart => "sheets",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Jacket => "png",
            Self::Audio => "wav",
            Self::Chart => "sus",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Jacket => "image/png",
            Self::Audio => "audio/wav",
            Self::Chart => "text/plain; charset=utf-8",
        }
    }
}

#[derive(Debug, Error)]
pub enum AssetUploadError {
    #[error("unsupported asset content type")]
    UnsupportedContentType,
    #[error("asset exceeds the size limit")]
    TooLarge,
    #[error("uploaded object is invalid")]
    InvalidData,
    #[error("asset file name must end with .sus")]
    InvalidFileName,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AssetUpload {
    pub bytes: Vec<u8>,
}

impl AssetUpload {
    pub fn jacket(content_type: &str, bytes: Vec<u8>) -> Result<Self, AssetUploadError> {
        if !matches!(content_type, "image/jpeg" | "image/png" | "image/webp") {
            return Err(AssetUploadError::UnsupportedContentType);
        }
        if bytes.len() > MAX_JACKET_SIZE {
            return Err(AssetUploadError::TooLarge);
        }
        let mut reader = ImageReader::new(Cursor::new(bytes));
        let mut limits = Limits::default();
        limits.max_image_width = Some(4096);
        limits.max_image_height = Some(4096);
        limits.max_alloc = Some(64 * 1024 * 1024);
        reader.limits(limits);
        let image = reader
            .with_guessed_format()
            .map_err(|_| AssetUploadError::InvalidData)?
            .decode()
            .map_err(|_| AssetUploadError::InvalidData)?;
        let mut normalized = Cursor::new(Vec::new());
        image
            .write_to(&mut normalized, ImageFormat::Png)
            .map_err(|_| AssetUploadError::InvalidData)?;
        if normalized.get_ref().len() > MAX_JACKET_SIZE {
            return Err(AssetUploadError::TooLarge);
        }
        Ok(Self {
            bytes: normalized.into_inner(),
        })
    }

    pub fn audio(content_type: &str, bytes: Vec<u8>) -> Result<Self, AssetUploadError> {
        if !matches!(content_type, "audio/wav" | "audio/wave" | "audio/x-wav") {
            return Err(AssetUploadError::UnsupportedContentType);
        }
        if bytes.len() > MAX_AUDIO_SIZE {
            return Err(AssetUploadError::TooLarge);
        }
        if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err(AssetUploadError::InvalidData);
        }
        Ok(Self { bytes })
    }

    pub fn chart(file_name: &str, bytes: Vec<u8>) -> Result<Self, AssetUploadError> {
        if !file_name.to_ascii_lowercase().ends_with(".sus") {
            return Err(AssetUploadError::InvalidFileName);
        }
        if bytes.len() > MAX_CHART_SIZE {
            return Err(AssetUploadError::TooLarge);
        }
        Ok(Self { bytes })
    }
}

pub struct AssetDownload {
    pub reader: Pin<Box<dyn AsyncRead + Send>>,
    pub content_length: u64,
    pub content_range: Option<String>,
}

pub trait AssetStorage: Send + Sync {
    fn upload<'a>(
        &'a self,
        kind: AssetKind,
        owner_id: &'a str,
        asset: AssetUpload,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send + 'a>>;

    fn delete<'a>(
        &'a self,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn download<'a>(
        &'a self,
        key: &'a str,
        range: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<AssetDownload>> + Send + 'a>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_wav_header() {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0; 4]);
        bytes.extend_from_slice(b"WAVE");

        assert_eq!(
            AssetUpload::audio("audio/wav", bytes).unwrap().bytes.len(),
            12
        );
    }

    #[test]
    fn rejects_invalid_wav_header() {
        assert!(matches!(
            AssetUpload::audio("audio/wav", b"invalid".to_vec()),
            Err(AssetUploadError::InvalidData)
        ));
    }

    #[test]
    fn requires_sus_chart_extension() {
        assert!(AssetUpload::chart("chart.SUS", Vec::new()).is_ok());
        assert!(matches!(
            AssetUpload::chart("chart.txt", Vec::new()),
            Err(AssetUploadError::InvalidFileName)
        ));
    }
}
