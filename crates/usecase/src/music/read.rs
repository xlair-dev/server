use domain::repository::{
    Repositories,
    music::{MusicListCursor, MusicRepository},
};

use super::{MusicPageDto, MusicUsecase, MusicUsecaseError};
use crate::model::music::MusicWithSheetsDto;

impl<R: Repositories> MusicUsecase<R> {
    pub async fn list_all(&self) -> Result<Vec<MusicWithSheetsDto>, MusicUsecaseError> {
        let musics = self.repositories.music().list_with_sheets().await?;
        Ok(musics.into_iter().map(MusicWithSheetsDto::from).collect())
    }

    pub async fn list_page(
        &self,
        cursor: Option<MusicListCursor>,
        limit: u64,
    ) -> Result<MusicPageDto, MusicUsecaseError> {
        let page = self
            .repositories
            .music()
            .list_with_sheets_page(cursor, limit)
            .await?;
        Ok(MusicPageDto {
            items: page.items.into_iter().map(Into::into).collect(),
            next_cursor: page.next_cursor,
        })
    }

    pub async fn find_by_id(
        &self,
        music_id: String,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        let music = self
            .repositories
            .music()
            .find_with_sheets(&music_id)
            .await?;
        Ok(music.into())
    }
}
