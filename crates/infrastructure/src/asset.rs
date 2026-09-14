use std::{future::Future, pin::Pin};

use aws_sdk_s3::{Client, primitives::ByteStream};
use sha2::{Digest, Sha256};
use usecase::asset::{AssetDownload, AssetKind, AssetStorage, AssetUpload};

#[derive(Clone)]
pub struct R2AssetStorage {
    client: Client,
    bucket: String,
}

impl R2AssetStorage {
    pub async fn new(
        endpoint: String,
        bucket: String,
        access_key_id: String,
        secret_access_key: String,
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
        let config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true)
            .build();
        Self {
            client: Client::from_conf(config),
            bucket,
        }
    }

    fn key(kind: AssetKind, owner_id: &str, bytes: &[u8]) -> String {
        let hash = Sha256::digest(bytes);
        format!(
            "{}/{}/{}.{}",
            kind.prefix(),
            owner_id,
            hex::encode(hash),
            kind.extension()
        )
    }
}

impl AssetStorage for R2AssetStorage {
    fn upload<'a>(
        &'a self,
        kind: AssetKind,
        owner_id: &'a str,
        asset: AssetUpload,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send + 'a>> {
        Box::pin(async move {
            let key = Self::key(kind, owner_id, &asset.bytes);
            self.client
                .put_object()
                .bucket(&self.bucket)
                .key(&key)
                .content_type(kind.content_type())
                .body(ByteStream::from(asset.bytes))
                .send()
                .await?;
            Ok(key)
        })
    }

    fn delete<'a>(
        &'a self,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.client
                .delete_object()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await?;
            Ok(())
        })
    }

    fn download<'a>(
        &'a self,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<AssetDownload>> + Send + 'a>> {
        Box::pin(async move {
            let object = self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await?;
            Ok(AssetDownload {
                reader: Box::pin(object.body.into_async_read()),
            })
        })
    }
}
