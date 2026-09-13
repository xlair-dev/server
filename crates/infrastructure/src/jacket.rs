use std::{future::Future, pin::Pin};

use aws_sdk_s3::{Client, primitives::ByteStream};
use sha2::{Digest, Sha256};
use usecase::jacket::{JacketStorage as JacketStoragePort, JacketUpload};

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

    fn key(music_id: &str, jacket: &JacketUpload) -> String {
        let hash = Sha256::digest(&jacket.bytes);
        format!("jackets/{music_id}/{hash:x}.png")
    }

    async fn upload_impl(&self, music_id: &str, jacket: JacketUpload) -> anyhow::Result<String> {
        let key = Self::key(music_id, &jacket);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .content_type("image/png")
            .body(ByteStream::from(jacket.bytes))
            .send()
            .await?;

        Ok(format!("{}/{}", self.public_base_url, key))
    }

    async fn delete_impl(&self, jacket_url: &str) -> anyhow::Result<()> {
        let prefix = format!("{}/", self.public_base_url);
        let key = jacket_url
            .strip_prefix(&prefix)
            .filter(|key| key.starts_with("jackets/") && key.ends_with(".png"))
            .ok_or_else(|| anyhow::anyhow!("invalid jacket URL: {jacket_url}"))?;
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
    fn upload<'a>(
        &'a self,
        music_id: &'a str,
        jacket: JacketUpload,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send + 'a>> {
        Box::pin(self.upload_impl(music_id, jacket))
    }

    fn delete<'a>(
        &'a self,
        jacket_url: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(self.delete_impl(jacket_url))
    }
}
