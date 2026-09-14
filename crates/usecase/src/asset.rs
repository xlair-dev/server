use std::{future::Future, io::Cursor, pin::Pin, str::FromStr};

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
    pub content_range: Option<AssetContentRange>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AssetRange {
    FromTo { start: u64, end: u64 },
    From { start: u64 },
    Suffix { length: u64 },
}

impl FromStr for AssetRange {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.strip_prefix("bytes=").ok_or(())?;
        if value.contains(',') {
            return Err(());
        }
        let (start, end) = value.split_once('-').ok_or(())?;
        if start.is_empty() {
            let length = end.parse().map_err(|_| ())?;
            return (length > 0).then_some(Self::Suffix { length }).ok_or(());
        }
        let start = start.parse().map_err(|_| ())?;
        if end.is_empty() {
            return Ok(Self::From { start });
        }
        let end = end.parse().map_err(|_| ())?;
        (start <= end)
            .then_some(Self::FromTo { start, end })
            .ok_or(())
    }
}

impl AssetRange {
    pub fn to_s3_header(self) -> String {
        match self {
            Self::FromTo { start, end } => format!("bytes={start}-{end}"),
            Self::From { start } => format!("bytes={start}-"),
            Self::Suffix { length } => format!("bytes=-{length}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct AssetContentRange {
    pub start: u64,
    pub end: u64,
    pub total: u64,
}

impl FromStr for AssetContentRange {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.strip_prefix("bytes ").ok_or(())?;
        let (range, total) = value.split_once('/').ok_or(())?;
        let (start, end) = range.split_once('-').ok_or(())?;
        let start = start.parse().map_err(|_| ())?;
        let end = end.parse().map_err(|_| ())?;
        let total = total.parse().map_err(|_| ())?;
        (start <= end && end < total)
            .then_some(Self { start, end, total })
            .ok_or(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssetDownloadError {
    #[error("asset range is not satisfiable")]
    RangeNotSatisfiable,
    #[error(transparent)]
    Storage(#[from] anyhow::Error),
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
        range: Option<AssetRange>,
    ) -> Pin<Box<dyn Future<Output = Result<AssetDownload, AssetDownloadError>> + Send + 'a>>;
}

#[cfg(test)]
mod range_tests {
    use std::str::FromStr;

    use super::{AssetContentRange, AssetRange};

    #[test]
    fn parses_supported_ranges() {
        assert_eq!(
            AssetRange::from_str("bytes=10-20"),
            Ok(AssetRange::FromTo { start: 10, end: 20 })
        );
        assert_eq!(
            AssetRange::from_str("bytes=10-"),
            Ok(AssetRange::From { start: 10 })
        );
        assert_eq!(
            AssetRange::from_str("bytes=-20"),
            Ok(AssetRange::Suffix { length: 20 })
        );
    }

    #[test]
    fn rejects_unsupported_ranges() {
        for value in ["10-20", "bytes=20-10", "bytes=-0", "bytes=1-2,4-5"] {
            assert!(AssetRange::from_str(value).is_err());
        }
    }

    #[test]
    fn parses_content_range() {
        assert_eq!(
            AssetContentRange::from_str("bytes 10-20/100"),
            Ok(AssetContentRange {
                start: 10,
                end: 20,
                total: 100
            })
        );
    }
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
