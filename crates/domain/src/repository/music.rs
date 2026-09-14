use std::future::Future;

use chrono::{DateTime, Utc};
use mockall::automock;
use thiserror::Error;

use crate::entity::{music::Music, sheet::Sheet};

#[derive(Debug, Error)]
pub enum MusicRepositoryError {
    #[error("Invalid music page limit: {0}")]
    InvalidLimit(u64),
    #[error("Music not found: {0}")]
    NotFound(String),
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

    fn find_with_sheets(
        &self,
        music_id: &str,
    ) -> impl Future<Output = Result<MusicWithSheets, MusicRepositoryError>> + Send;

    fn find_sheet(
        &self,
        sheet_id: &str,
    ) -> impl Future<Output = Result<Sheet, MusicRepositoryError>> + Send;

    fn insert_with_sheets(
        &self,
        music: MusicWithSheets,
    ) -> impl Future<Output = Result<MusicWithSheets, MusicRepositoryError>> + Send;

    fn update_with_sheets(
        &self,
        music: MusicWithSheets,
    ) -> impl Future<Output = Result<MusicWithSheets, MusicRepositoryError>> + Send;

    fn update_jacket_key(
        &self,
        music_id: &str,
        jacket_key: Option<String>,
    ) -> impl Future<Output = Result<MusicWithSheets, MusicRepositoryError>> + Send;

    fn update_music_key(
        &self,
        music_id: &str,
        music_key: Option<String>,
    ) -> impl Future<Output = Result<MusicWithSheets, MusicRepositoryError>> + Send;

    fn update_chart_key(
        &self,
        sheet_id: &str,
        chart_key: Option<String>,
    ) -> impl Future<Output = Result<MusicWithSheets, MusicRepositoryError>> + Send;
}
