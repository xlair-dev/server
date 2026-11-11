use chrono::Utc;
use domain::{
    entity::user_play_option::UserPlayOption,
    repository::{Repositories, user::UserRepository},
};
use tracing::{debug, instrument};

use crate::{
    model::user::{UserPlayOptionDto, UserPlayOptionUpdateDto},
    user::{UserUsecase, UserUsecaseError},
};

impl<R: Repositories> UserUsecase<R> {
    /// Falls back to the cabinet firmware's default note speed when no row exists.
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_play_option(
        &self,
        user_id: String,
    ) -> Result<UserPlayOptionDto, UserUsecaseError> {
        debug!("Loading user aggregate to verify existence");
        self.repositories.user().find_by_id(&user_id).await?.ok_or(
            UserUsecaseError::NotFoundById {
                user_id: user_id.clone(),
            },
        )?;

        debug!("Resolving user play option");
        let option = self.repositories.user().find_play_option(&user_id).await?;
        let dto = option
            .map(UserPlayOptionDto::from)
            .unwrap_or_else(|| UserPlayOptionDto::with_defaults(user_id));
        Ok(dto)
    }

    #[instrument(skip(self, update), fields(user_id = %user_id))]
    pub async fn save_play_option(
        &self,
        user_id: String,
        update: UserPlayOptionUpdateDto,
    ) -> Result<UserPlayOptionDto, UserUsecaseError> {
        fn valid_step(value: f32, min: f32, max: f32, steps_per_unit: f32) -> bool {
            value.is_finite()
                && (min..=max).contains(&value)
                && ((value * steps_per_unit).round() - value * steps_per_unit).abs() < 0.0001
        }
        if !valid_step(update.note_speed, 0.25, 10.0, 4.0) {
            return Err(UserUsecaseError::InvalidPlayOption("noteSpeed"));
        }
        if !(-100..=100).contains(&update.audio_judgment_timing) {
            return Err(UserUsecaseError::InvalidPlayOption("audioJudgmentTiming"));
        }
        if !valid_step(update.visual_judgment_timing, -10.0, 10.0, 10.0) {
            return Err(UserUsecaseError::InvalidPlayOption("visualJudgmentTiming"));
        }
        if update.note_volume > 100 {
            return Err(UserUsecaseError::InvalidPlayOption("noteVolume"));
        }
        debug!("Validating user existence prior to saving play option");
        self.repositories.user().find_by_id(&user_id).await?.ok_or(
            UserUsecaseError::NotFoundById {
                user_id: user_id.clone(),
            },
        )?;

        let mut option = UserPlayOption::new(user_id, update.note_speed, Utc::now());
        option.set_audio_judgment_timing(update.audio_judgment_timing);
        option.set_visual_judgment_timing(update.visual_judgment_timing);
        option.set_mirror(update.mirror);
        option.set_note_volume(update.note_volume);

        debug!("Persisting user play option");
        let saved = self.repositories.user().save_play_option(option).await?;
        Ok(UserPlayOptionDto::from(saved))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::TimeZone;
    use domain::{
        entity::user_play_option::UserPlayOption,
        repository::{
            MockRepositories,
            music::MockMusicRepository,
            record::MockRecordRepository,
            user::{MockUserRepository, UserRepositoryError},
        },
        testing::{datetime::sample_timestamp, user::USER1},
    };

    use super::*;

    fn user_sample() -> UserPlayOption {
        let updated_at = chrono::Utc
            .timestamp_opt(1_694_000_000, 0)
            .single()
            .expect("valid timestamp");
        UserPlayOption::new("user-123".to_owned(), 1.5, updated_at)
    }

    fn build_usecase(mut user_repo: MockUserRepository) -> UserUsecase<MockRepositories> {
        let repositories = MockRepositories {
            user: {
                user_repo.expect_find_by_id().returning(|_| {
                    let created_at = sample_timestamp();
                    let user = USER1.build(true, false, created_at);
                    Box::pin(async move { Ok(Some(user)) })
                });
                user_repo
            },
            record: MockRecordRepository::new(),
            music: MockMusicRepository::new(),
        };
        UserUsecase::new(Arc::new(repositories))
    }

    #[tokio::test]
    async fn get_play_option_returns_stored_value() {
        let mut user_repo = MockUserRepository::new();
        let sample = user_sample();
        user_repo
            .expect_find_play_option()
            .withf(|user_id| user_id == "user-123")
            .returning(move |_| {
                let sample = sample.clone();
                Box::pin(async move { Ok(Some(sample)) })
            });

        let usecase = build_usecase(user_repo);

        let result = usecase
            .get_play_option("user-123".to_owned())
            .await
            .expect("should succeed");

        assert_eq!(result.user_id, "user-123");
        assert!((result.note_speed - 1.5).abs() < f32::EPSILON);
        assert_eq!(result.audio_judgment_timing, 0);
    }

    #[tokio::test]
    async fn get_play_option_returns_defaults_when_missing() {
        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_play_option()
            .returning(|_| Box::pin(async { Ok(None) }));

        let usecase = build_usecase(user_repo);

        let result = usecase
            .get_play_option("user-123".to_owned())
            .await
            .expect("should succeed");

        assert_eq!(result.user_id, "user-123");
        assert!((result.note_speed - 1.0).abs() < f32::EPSILON);
        assert_eq!(result.audio_judgment_timing, 0);
    }

    #[tokio::test]
    async fn get_play_option_returns_not_found_when_user_missing() {
        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_by_id()
            .withf(|user_id| user_id == "missing-user")
            .returning(|_| Box::pin(async { Ok(None) }));

        user_repo.expect_find_play_option().never();

        let repositories = MockRepositories {
            user: user_repo,
            record: MockRecordRepository::new(),
            music: MockMusicRepository::new(),
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let err = usecase
            .get_play_option("missing-user".to_owned())
            .await
            .expect_err("should return not found");

        matches!(err, UserUsecaseError::NotFoundById { user_id } if user_id == "missing-user");
    }

    #[tokio::test]
    async fn save_play_option_persists_and_returns_saved_value() {
        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_play_option()
            .returning(|_| Box::pin(async { Ok(None) }));
        user_repo
            .expect_save_play_option()
            .withf(|option| {
                option.user_id() == "user-123"
                    && (option.note_speed() - 1.25).abs() < f32::EPSILON
                    && *option.note_volume() == 100
            })
            .returning(|option| Box::pin(async move { Ok(option) }));

        let usecase = build_usecase(user_repo);

        let update = UserPlayOptionUpdateDto {
            note_speed: 1.25,
            audio_judgment_timing: 0,
            visual_judgment_timing: 0.0,
            mirror: false,
            note_volume: 100,
        };
        let result = usecase
            .save_play_option("user-123".to_owned(), update)
            .await
            .expect("should succeed");

        assert_eq!(result.user_id, "user-123");
        assert!((result.note_speed - 1.25).abs() < f32::EPSILON);
        assert_eq!(result.note_volume, 100);
    }

    #[tokio::test]
    async fn save_play_option_rejects_values_outside_the_specified_steps_and_ranges() {
        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: MockRecordRepository::new(),
            music: MockMusicRepository::new(),
        };
        let usecase = UserUsecase::new(Arc::new(repositories));
        for update in [
            UserPlayOptionUpdateDto {
                note_speed: 0.0,
                audio_judgment_timing: 0,
                visual_judgment_timing: 0.0,
                mirror: false,
                note_volume: 100,
            },
            UserPlayOptionUpdateDto {
                note_speed: 1.2,
                audio_judgment_timing: 0,
                visual_judgment_timing: 0.0,
                mirror: false,
                note_volume: 100,
            },
            UserPlayOptionUpdateDto {
                note_speed: 1.0,
                audio_judgment_timing: -101,
                visual_judgment_timing: 0.0,
                mirror: false,
                note_volume: 100,
            },
            UserPlayOptionUpdateDto {
                note_speed: 1.0,
                audio_judgment_timing: 0,
                visual_judgment_timing: 0.05,
                mirror: false,
                note_volume: 100,
            },
            UserPlayOptionUpdateDto {
                note_speed: 1.0,
                audio_judgment_timing: 0,
                visual_judgment_timing: 0.0,
                mirror: false,
                note_volume: 101,
            },
        ] {
            assert!(matches!(
                usecase
                    .save_play_option("user-123".to_owned(), update)
                    .await,
                Err(UserUsecaseError::InvalidPlayOption(_))
            ));
        }
    }

    #[tokio::test]
    async fn save_play_option_returns_error_on_repository_failure() {
        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_play_option()
            .returning(|_| Box::pin(async { Ok(None) }));
        user_repo.expect_save_play_option().returning(|_| {
            Box::pin(async { Err(UserRepositoryError::InternalError(anyhow::anyhow!("boom"))) })
        });

        let usecase = build_usecase(user_repo);

        let update = UserPlayOptionUpdateDto {
            note_speed: 1.25,
            audio_judgment_timing: 0,
            visual_judgment_timing: 0.0,
            mirror: false,
            note_volume: 100,
        };
        let err = usecase
            .save_play_option("user-123".to_owned(), update)
            .await
            .expect_err("should propagate error");

        match err {
            UserUsecaseError::UserRepositoryError(UserRepositoryError::InternalError(inner)) => {
                assert_eq!(inner.to_string(), "boom");
            }
            _ => panic!("unexpected error variant"),
        }
    }
}
