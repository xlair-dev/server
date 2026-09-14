mod read;
mod read_adapter;
mod write;
mod write_adapter;

use std::sync::Arc;

use domain::repository::music::{
    MusicListCursor, MusicListPage, MusicRepository, MusicRepositoryError, MusicWithSheets,
};
use sea_orm::DbConn;
use tracing::{debug, info, instrument};

pub struct MusicRepositoryImpl {
    db: Arc<DbConn>,
}

impl MusicRepositoryImpl {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

impl MusicRepository for MusicRepositoryImpl {
    #[instrument(skip(self))]
    async fn list_with_sheets(&self) -> Result<Vec<MusicWithSheets>, MusicRepositoryError> {
        debug!("Loading music metadata via SeaORM");
        let musics = read::list_with_sheets(self.db.as_ref()).await?;
        info!(count = musics.len(), "Music metadata loaded");
        Ok(musics)
    }

    #[instrument(skip(self))]
    async fn list_with_sheets_page(
        &self,
        cursor: Option<MusicListCursor>,
        limit: u64,
    ) -> Result<MusicListPage, MusicRepositoryError> {
        debug!(limit, "Loading a page of music metadata via SeaORM");
        read::list_with_sheets_page(self.db.as_ref(), cursor, limit).await
    }

    #[instrument(skip(self), fields(music_id = %music_id))]
    async fn find_with_sheets(
        &self,
        music_id: &str,
    ) -> Result<MusicWithSheets, MusicRepositoryError> {
        debug!("Loading music metadata by id via SeaORM");
        read::find_with_sheets(self.db.as_ref(), music_id).await
    }

    async fn find_sheet(
        &self,
        sheet_id: &str,
    ) -> Result<domain::entity::sheet::Sheet, MusicRepositoryError> {
        read::find_sheet(self.db.as_ref(), sheet_id).await
    }

    #[instrument(skip(self), fields(music_id = %music.music.id()))]
    async fn insert_with_sheets(
        &self,
        music: MusicWithSheets,
    ) -> Result<MusicWithSheets, MusicRepositoryError> {
        write::insert_with_sheets(self.db.as_ref(), music).await
    }

    #[instrument(skip(self), fields(music_id = %music.music.id()))]
    async fn update_with_sheets(
        &self,
        music: MusicWithSheets,
    ) -> Result<MusicWithSheets, MusicRepositoryError> {
        write::update_with_sheets(self.db.as_ref(), music).await
    }

    #[instrument(skip(self), fields(music_id = %music_id))]
    async fn delete(&self, music_id: &str) -> Result<(), MusicRepositoryError> {
        write::delete(self.db.as_ref(), music_id).await
    }

    #[instrument(skip(self), fields(music_id = %music_id))]
    async fn update_jacket_key(
        &self,
        music_id: &str,
        jacket_key: Option<String>,
    ) -> Result<MusicWithSheets, MusicRepositoryError> {
        write::update_jacket_key(self.db.as_ref(), music_id, jacket_key).await
    }

    async fn update_audio_key(
        &self,
        music_id: &str,
        audio_key: Option<String>,
    ) -> Result<MusicWithSheets, MusicRepositoryError> {
        write::update_audio_key(self.db.as_ref(), music_id, audio_key).await
    }

    async fn update_chart_key(
        &self,
        sheet_id: &str,
        chart_key: Option<String>,
    ) -> Result<MusicWithSheets, MusicRepositoryError> {
        write::update_chart_key(self.db.as_ref(), sheet_id, chart_key).await
    }
}
