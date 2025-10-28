use std::future::Future;

use mockall::automock;
use thiserror::Error;

use crate::entity::{music::Music, sheet::Sheet};

#[derive(Debug, Error)]
pub enum MusicRepositoryError {
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct MusicWithSheets {
    pub music: Music,
    pub sheets: Vec<Sheet>,
}

impl MusicWithSheets {
    pub fn new(music: Music, sheets: Vec<Sheet>) -> Self {
        Self { music, sheets }
    }
}

#[automock]
pub trait MusicRepository: Send + Sync {
    fn list_with_sheets(
        &self,
    ) -> impl Future<Output = Result<Vec<MusicWithSheets>, MusicRepositoryError>> + Send;
}
