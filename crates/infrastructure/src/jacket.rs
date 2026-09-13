use std::{future::Future, pin::Pin, time::Duration};

use aws_sdk_s3::{Client, presigning::PresigningConfig};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use usecase::jacket::{JacketStorage as JacketStoragePort, JacketUpload};

const UPLOAD_EXPIRATION: Duration = Duration::from_secs(15 * 60);
const MAX_JACKET_SIZE: i64 = 5 * 1024 * 1024;

#[derive(Clone)]
pub struct R2JacketStorage {
    client: Client,
    bucket: String,
    public_base_url: String,
    upload_secret: Vec<u8>,
}

impl R2JacketStorage {
    pub async fn new(
        endpoint: String,
        bucket: String,
        access_key_id: String,
        secret_access_key: String,
        public_base_url: String,
        upload_secret: String,
    ) -> Self {
        let credentials = aws_sdk_s3::config::Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "xlair-r2",
        );
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(aws_sdk_s3::config::Region::new("auto"))
            .credentials_provider(credentials)
            .load()
            .await;
        Self {
            client: Client::new(&config),
            bucket,
            public_base_url: public_base_url.trim_end_matches('/').to_owned(),
            upload_secret: upload_secret.into_bytes(),
        }
    }

    fn key(upload_id: &str, content_type: &str) -> anyhow::Result<String> {
        let extension = match content_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            _ => anyhow::bail!("unsupported jacket content type"),
        };
        Ok(format!("jackets/{upload_id}.{extension}"))
    }

    fn cleanup_token(&self, upload_id: &str, content_type: &str) -> anyhow::Result<String> {
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.upload_secret)?;
        mac.update(upload_id.as_bytes());
        mac.update(b":");
        mac.update(content_type.as_bytes());
        Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
    }

    fn verify_cleanup_token(
        &self,
        upload_id: &str,
        content_type: &str,
        token: &str,
    ) -> anyhow::Result<()> {
        let token = URL_SAFE_NO_PAD.decode(token)?;
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.upload_secret)?;
        mac.update(upload_id.as_bytes());
        mac.update(b":");
        mac.update(content_type.as_bytes());
        mac.verify_slice(&token)
            .map_err(|_| anyhow::anyhow!("cleanup token is invalid"))
    }

    async fn create_upload_url_impl(
        &self,
        upload_id: &str,
        content_type: &str,
    ) -> anyhow::Result<JacketUpload> {
        let key = Self::key(upload_id, content_type)?;
        let config = PresigningConfig::expires_in(UPLOAD_EXPIRATION)
            .map_err(|error| anyhow::anyhow!("failed to create presigning config: {error}"))?;
        let request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .content_type(content_type)
            .presigned(config)
            .await
            .map_err(|error| anyhow::anyhow!("failed to create presigned URL: {error}"))?;
        Ok(JacketUpload {
            upload_url: request.uri().to_owned(),
            jacket_url: format!("{}/{}", self.public_base_url, key),
            upload_id: upload_id.to_owned(),
            cleanup_token: self.cleanup_token(upload_id, content_type)?,
        })
    }

    async fn validate_upload_impl(
        &self,
        upload_id: &str,
        content_type: &str,
        cleanup_token: &str,
    ) -> anyhow::Result<()> {
        self.verify_cleanup_token(upload_id, content_type, cleanup_token)?;
        let key = Self::key(upload_id, content_type)?;
        let metadata = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await?;
        if metadata.content_length.unwrap_or(0) > MAX_JACKET_SIZE {
            anyhow::bail!("jacket image exceeds the 5 MiB limit");
        }
        let object = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        let bytes = object.body.collect().await?.into_bytes();
        if bytes.len() as i64 > MAX_JACKET_SIZE {
            anyhow::bail!("jacket image exceeds the 5 MiB limit");
        }
        let valid = match content_type {
            "image/jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
            "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            "image/webp" => bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
            _ => false,
        };
        if !valid {
            anyhow::bail!("uploaded object is not a valid {content_type} image");
        }
        Ok(())
    }

    async fn delete_upload_impl(
        &self,
        upload_id: &str,
        content_type: &str,
        cleanup_token: &str,
    ) -> anyhow::Result<()> {
        self.verify_cleanup_token(upload_id, content_type, cleanup_token)?;
        let key = Self::key(upload_id, content_type)?;
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}

impl JacketStoragePort for R2JacketStorage {
    fn create_upload_url<'a>(
        &'a self,
        upload_id: &'a str,
        content_type: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<JacketUpload>> + Send + 'a>> {
        Box::pin(self.create_upload_url_impl(upload_id, content_type))
    }

    fn validate_upload<'a>(
        &'a self,
        upload_id: &'a str,
        content_type: &'a str,
        cleanup_token: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(self.validate_upload_impl(upload_id, content_type, cleanup_token))
    }

    fn delete_upload<'a>(
        &'a self,
        upload_id: &'a str,
        content_type: &'a str,
        cleanup_token: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(self.delete_upload_impl(upload_id, content_type, cleanup_token))
    }
}
