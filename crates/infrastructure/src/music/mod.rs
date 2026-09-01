mod adapter;
mod read;

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
}
