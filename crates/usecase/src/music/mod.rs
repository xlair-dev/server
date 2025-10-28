use std::sync::Arc;

use domain::repository::{
    Repositories,
    music::{MusicRepository, MusicRepositoryError, MusicWithSheets},
};
use thiserror::Error;

use crate::model::music::MusicWithSheetsDto;

#[derive(Debug, Error)]
pub enum MusicUsecaseError {
    #[error(transparent)]
    MusicRepository(#[from] MusicRepositoryError),
}

pub struct MusicUsecase<R: Repositories> {
    repositories: Arc<R>,
}

impl<R: Repositories> MusicUsecase<R> {
    pub fn new(repositories: Arc<R>) -> Self {
        Self { repositories }
    }

    pub async fn list_all(&self) -> Result<Vec<MusicWithSheetsDto>, MusicUsecaseError> {
        let musics = self.repositories.music().list_with_sheets().await?;
        Ok(musics.into_iter().map(MusicWithSheetsDto::from).collect())
    }
}

impl<R: Repositories> Clone for MusicUsecase<R> {
    fn clone(&self) -> Self {
        Self {
            repositories: Arc::clone(&self.repositories),
        }
    }
}

impl From<MusicWithSheets> for MusicWithSheetsDto {
    fn from(value: MusicWithSheets) -> Self {
        MusicWithSheetsDto::new(
            value.music.into(),
            value.sheets.into_iter().map(Into::into).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use domain::{
        entity::{difficulty::Difficulty, genre::Genre, level::Level, music::Music, sheet::Sheet},
        repository::{
            MockRepositories,
            music::{MockMusicRepository, MusicWithSheets},
            record::MockRecordRepository,
            user::MockUserRepository,
        },
    };

    use super::*;

    #[tokio::test]
    async fn list_all_returns_entries() {
        let mut music_repo = MockMusicRepository::new();
        music_repo.expect_list_with_sheets().returning(|| {
            let music = Music::new(
                "music-1".to_owned(),
                "Song".to_owned(),
                "Artist".to_owned(),
                135.5,
                Genre::ORIGINAL,
                "jacket.png".to_owned(),
                Utc::now(),
                false,
            );
            let sheet = Sheet::new(
                "sheet-1".to_owned(),
                "music-1".to_owned(),
                Difficulty::Easy,
                Level::new(12, 3).expect("level"),
                "Designer".to_owned(),
            );
            Box::pin(async move { Ok(vec![MusicWithSheets::new(music, vec![sheet])]) })
        });

        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: MockRecordRepository::new(),
            music: music_repo,
        };
        let usecase = MusicUsecase::new(Arc::new(repositories));

        let result = usecase.list_all().await.expect("should succeed");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].music.id, "music-1");
        assert_eq!(result[0].sheets.len(), 1);
        assert_eq!(result[0].sheets[0].id, "sheet-1");
    }
}
