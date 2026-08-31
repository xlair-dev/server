use std::future::Future;

use chrono::{DateTime, Utc};
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MusicListCursor {
    pub registration_date: DateTime<Utc>,
    pub id: String,
}

#[derive(Debug)]
pub struct MusicListPage {
    pub items: Vec<MusicWithSheets>,
    pub next_cursor: Option<MusicListCursor>,
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

    fn list_with_sheets_page(
        &self,
        cursor: Option<MusicListCursor>,
        limit: u64,
    ) -> impl Future<Output = Result<MusicListPage, MusicRepositoryError>> + Send;
}
