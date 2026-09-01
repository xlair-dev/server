use std::{collections::HashSet, sync::Arc};

use domain::{
    entity::{difficulty::Difficulty, genre::Genre, level::Level, music::Music, sheet::Sheet},
    repository::{
        Repositories,
        music::{MusicListCursor, MusicRepository, MusicRepositoryError, MusicWithSheets},
    },
};
use thiserror::Error;

use crate::model::music::{
    CreateMusicInput, MusicDataInput, MusicWithSheetsDto, SheetDataInput, SheetInput,
    UpdateMusicInput,
};

#[derive(Debug, Error)]
pub enum MusicUsecaseError {
    #[error(transparent)]
    MusicRepository(#[from] MusicRepositoryError),
    #[error("Invalid music input: {0}")]
    InvalidInput(String),
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

    pub async fn create(
        &self,
        input: CreateMusicInput,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        let music = build_music(
            input.music,
            uuid::Uuid::new_v4().to_string(),
            input.sheets.into_iter().map(Into::into).collect(),
            None,
        )?;
        let created = self.repositories.music().insert_with_sheets(music).await?;
        Ok(created.into())
    }

    pub async fn update(
        &self,
        music_id: String,
        input: UpdateMusicInput,
    ) -> Result<MusicWithSheetsDto, MusicUsecaseError> {
        if uuid::Uuid::parse_str(&music_id).is_err() {
            return Err(MusicUsecaseError::InvalidInput(
                "music id is invalid".to_owned(),
            ));
        }
        let existing = self
            .repositories
            .music()
            .find_with_sheets(&music_id)
            .await?;
        let existing_sheet_ids: HashSet<&str> = existing
            .sheets
            .iter()
            .map(|sheet| sheet.id().as_str())
            .collect();
        let requested_sheet_ids: HashSet<&str> =
            input.sheets.iter().map(|sheet| sheet.id.as_str()).collect();
        if existing_sheet_ids != requested_sheet_ids {
            return Err(MusicUsecaseError::InvalidInput(
                "sheet ids must match the existing sheets".to_owned(),
            ));
        }
        let music = build_music(
            input.music,
            music_id,
            input.sheets.into_iter().map(Into::into).collect(),
            Some(existing),
        )?;
        let updated = self.repositories.music().update_with_sheets(music).await?;
        Ok(updated.into())
    }
}

fn build_music(
    input: MusicDataInput,
    music_id: String,
    sheets_input: Vec<SheetBuildInput>,
    existing: Option<MusicWithSheets>,
) -> Result<MusicWithSheets, MusicUsecaseError> {
    if input.title.trim().is_empty()
        || input.artist.trim().is_empty()
        || input.jacket.trim().is_empty()
        || !input.bpm.is_finite()
        || input.bpm <= 0.0
    {
        return Err(MusicUsecaseError::InvalidInput(
            "title, artist, jacket, and bpm must be valid".to_owned(),
        ));
    }
    if input.genre != "ORIGINAL" {
        return Err(MusicUsecaseError::InvalidInput(
            "genre is invalid".to_owned(),
        ));
    }
    if sheets_input.len() != 3 {
        return Err(MusicUsecaseError::InvalidInput(
            "exactly one sheet for each difficulty is required".to_owned(),
        ));
    }

    let mut sheets = Vec::with_capacity(3);
    let mut seen = [false; 3];
    for sheet in sheets_input {
        let difficulty = match sheet.data.difficulty.as_str() {
            "easy" => {
                if seen[0] {
                    return invalid_sheet();
                }
                seen[0] = true;
                Difficulty::Easy
            }
            "normal" => {
                if seen[1] {
                    return invalid_sheet();
                }
                seen[1] = true;
                Difficulty::Normal
            }
            "hard" => {
                if seen[2] {
                    return invalid_sheet();
                }
                seen[2] = true;
                Difficulty::Hard
            }
            _ => return invalid_sheet(),
        };
        let level = level_from_value(sheet.data.level)?;
        let id = match (&existing, sheet.id) {
            (None, None) => uuid::Uuid::new_v4().to_string(),
            (Some(_), Some(id)) if uuid::Uuid::parse_str(&id).is_ok() => id,
            _ => return invalid_sheet(),
        };
        sheets.push(Sheet::new(
            id,
            music_id.clone(),
            difficulty,
            level,
            non_empty(sheet.data.notes_designer, "notesDesigner")?,
        ));
    }
    if seen != [true; 3] {
        return invalid_sheet();
    }
    Ok(MusicWithSheets::new(
        Music::new(
            music_id,
            input.title,
            input.artist,
            input.bpm,
            Genre::ORIGINAL,
            input.jacket,
            input.registration_date,
            input.is_test,
        ),
        sheets,
    ))
}

struct SheetBuildInput {
    id: Option<String>,
    data: SheetDataInput,
}

impl From<SheetDataInput> for SheetBuildInput {
    fn from(data: SheetDataInput) -> Self {
        Self { id: None, data }
    }
}

impl From<SheetInput> for SheetBuildInput {
    fn from(value: SheetInput) -> Self {
        Self {
            id: Some(value.id),
            data: SheetDataInput {
                difficulty: value.difficulty,
                level: value.level,
                notes_designer: value.notes_designer,
            },
        }
    }
}

fn invalid_sheet<T>() -> Result<T, MusicUsecaseError> {
    Err(MusicUsecaseError::InvalidInput(
        "sheets are invalid".to_owned(),
    ))
}

fn non_empty(value: String, field: &str) -> Result<String, MusicUsecaseError> {
    if value.trim().is_empty() {
        return Err(MusicUsecaseError::InvalidInput(format!(
            "{field} must not be empty"
        )));
    }
    Ok(value)
}

fn level_from_value(value: f64) -> Result<Level, MusicUsecaseError> {
    if !value.is_finite() || value < 1.0 || value > 99.9 {
        return Err(MusicUsecaseError::InvalidInput(
            "sheet level is invalid".to_owned(),
        ));
    }
    let scaled = (value * 10.0).round();
    if (scaled / 10.0 - value).abs() > f64::EPSILON {
        return Err(MusicUsecaseError::InvalidInput(
            "sheet level is invalid".to_owned(),
        ));
    }
    Level::new((scaled as u32) / 10, (scaled as u32) % 10)
        .map_err(|_| MusicUsecaseError::InvalidInput("sheet level is invalid".to_owned()))
}

pub struct MusicPageDto {
    pub items: Vec<MusicWithSheetsDto>,
    pub next_cursor: Option<MusicListCursor>,
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

    use chrono::{TimeZone, Utc};
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
    use crate::model::music::{CreateMusicInput, MusicDataInput, SheetDataInput};

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

    #[tokio::test]
    async fn find_by_id_returns_entry() {
        let mut music_repo = MockMusicRepository::new();
        music_repo
            .expect_find_with_sheets()
            .withf(|music_id| music_id == "music-1")
            .returning(|_| {
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
                Box::pin(async move { Ok(MusicWithSheets::new(music, Vec::new())) })
            });

        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: MockRecordRepository::new(),
            music: music_repo,
        };
        let usecase = MusicUsecase::new(Arc::new(repositories));

        let result = usecase
            .find_by_id("music-1".to_owned())
            .await
            .expect("should succeed");
        assert_eq!(result.music.id, "music-1");
        assert!(result.sheets.is_empty());
    }

    fn write_input() -> CreateMusicInput {
        CreateMusicInput {
            music: MusicDataInput {
                title: "Song".to_owned(),
                artist: "Artist".to_owned(),
                bpm: 135.5,
                genre: "ORIGINAL".to_owned(),
                jacket: "jacket.png".to_owned(),
                registration_date: Utc.with_ymd_and_hms(2025, 10, 1, 12, 0, 0).unwrap(),
                is_test: false,
            },
            sheets: vec![
                SheetDataInput {
                    difficulty: "easy".to_owned(),
                    level: 12.3,
                    notes_designer: "Easy Designer".to_owned(),
                },
                SheetDataInput {
                    difficulty: "normal".to_owned(),
                    level: 13.0,
                    notes_designer: "Normal Designer".to_owned(),
                },
                SheetDataInput {
                    difficulty: "hard".to_owned(),
                    level: 14.7,
                    notes_designer: "Hard Designer".to_owned(),
                },
            ],
        }
    }

    #[tokio::test]
    async fn create_generates_ids_for_music_and_sheets() {
        let mut music_repo = MockMusicRepository::new();
        music_repo
            .expect_insert_with_sheets()
            .returning(|music| Box::pin(async move { Ok(music) }));

        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: MockRecordRepository::new(),
            music: music_repo,
        };
        let usecase = MusicUsecase::new(Arc::new(repositories));

        let result = usecase.create(write_input()).await.expect("should succeed");
        assert!(uuid::Uuid::parse_str(&result.music.id).is_ok());
        assert_eq!(result.sheets.len(), 3);
        assert!(
            result
                .sheets
                .iter()
                .all(|sheet| uuid::Uuid::parse_str(&sheet.id).is_ok())
        );
    }

    #[tokio::test]
    async fn create_rejects_duplicate_difficulties() {
        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: MockRecordRepository::new(),
            music: MockMusicRepository::new(),
        };
        let usecase = MusicUsecase::new(Arc::new(repositories));
        let mut input = write_input();
        input.sheets[1].difficulty = "easy".to_owned();

        assert!(matches!(
            usecase.create(input).await,
            Err(MusicUsecaseError::InvalidInput(_))
        ));
    }
}
