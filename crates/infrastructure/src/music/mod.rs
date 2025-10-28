mod adapter;
mod read;

use std::sync::Arc;

use domain::repository::music::{MusicRepository, MusicRepositoryError, MusicWithSheets};
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
}
