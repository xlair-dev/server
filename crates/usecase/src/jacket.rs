use std::{future::Future, pin::Pin};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JacketUpload {
    pub upload_url: String,
    pub jacket_url: String,
    pub upload_id: String,
}

pub trait JacketStorage: Send + Sync {
    fn create_upload_url<'a>(
        &'a self,
        upload_id: &'a str,
        content_type: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<JacketUpload>> + Send + 'a>>;

    fn validate_upload<'a>(
        &'a self,
        upload_id: &'a str,
        content_type: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

    fn delete_upload<'a>(
        &'a self,
        upload_id: &'a str,
        content_type: &'a str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;
}
