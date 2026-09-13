use std::{future::Future, pin::Pin};

use aws_sdk_s3::{Client, primitives::ByteStream};
use usecase::jacket::{JacketStorage as JacketStoragePort, JacketUpload, UploadedJacket};

#[derive(Clone)]
pub struct R2JacketStorage {
    client: Client,
    bucket: String,
    public_base_url: String,
}

impl R2JacketStorage {
    pub async fn new(
        endpoint: String,
        bucket: String,
        access_key_id: String,
        secret_access_key: String,
        public_base_url: String,
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
        }
    }

    fn extension(content_type: &str) -> anyhow::Result<&'static str> {
        match content_type {
            "image/jpeg" => Ok("jpg"),
            "image/png" => Ok("png"),
            "image/webp" => Ok("webp"),
            _ => anyhow::bail!("unsupported jacket content type"),
        }
    }

    fn key(music_id: &str, content_type: &str) -> anyhow::Result<String> {
        Ok(format!(
            "jackets/{music_id}.{}",
            Self::extension(content_type)?
        ))
    }

    async fn upload_impl(
        &self,
        music_id: &str,
        jacket: JacketUpload,
    ) -> anyhow::Result<UploadedJacket> {
        let key = Self::key(music_id, &jacket.content_type)?;
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .content_type(&jacket.content_type)
            .body(ByteStream::from(jacket.bytes))
            .send()
            .await?;

        Ok(UploadedJacket {
            url: format!("{}/{}", self.public_base_url, key),
        })
    }

    async fn delete_impl(&self, music_id: &str) -> anyhow::Result<()> {
        for extension in ["jpg", "png", "webp"] {
            self.client
                .delete_object()
                .bucket(&self.bucket)
                .key(format!("jackets/{music_id}.{extension}"))
                .send()
                .await?;
        }
        Ok(())
    }
}

impl JacketStoragePort for R2JacketStorage {
    fn upload<'a>(
        &'a self,
        music_id: &'a str,
        jacket: JacketUpload,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<UploadedJacket>> + Send + 'a>> {
        Box::pin(self.upload_impl(music_id, jacket))
    }

    fn delete<'a>(
        &'a self,
        music_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(self.delete_impl(music_id))
    }
}
