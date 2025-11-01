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
    /// Falls back to firmware defaults (noteSpeed=1.0, judgmentOffset=0) when no rows exist yet.
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
        debug!("Validating user existence prior to saving play option");
        self.repositories.user().find_by_id(&user_id).await?.ok_or(
            UserUsecaseError::NotFoundById {
                user_id: user_id.clone(),
            },
        )?;

        let option = UserPlayOption::new(
            user_id.clone(),
            update.note_speed,
            update.judgment_offset,
            Utc::now(),
        );

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
        UserPlayOption::new("user-123".to_owned(), 1.5, -5, updated_at)
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
        assert_eq!(result.judgment_offset, -5);
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
        assert_eq!(result.judgment_offset, 0);
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
            .expect_save_play_option()
            .withf(|option| {
                option.user_id() == "user-123"
                    && (option.note_speed() - 1.25).abs() < f32::EPSILON
                    && *option.judgment_offset() == 3
            })
            .returning(|option| Box::pin(async move { Ok(option) }));

        let usecase = build_usecase(user_repo);

        let update = UserPlayOptionUpdateDto::new(1.25, 3);
        let result = usecase
            .save_play_option("user-123".to_owned(), update)
            .await
            .expect("should succeed");

        assert_eq!(result.user_id, "user-123");
        assert!((result.note_speed - 1.25).abs() < f32::EPSILON);
        assert_eq!(result.judgment_offset, 3);
    }

    #[tokio::test]
    async fn save_play_option_returns_error_on_repository_failure() {
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_save_play_option().returning(|_| {
            Box::pin(async { Err(UserRepositoryError::InternalError(anyhow::anyhow!("boom"))) })
        });

        let usecase = build_usecase(user_repo);

        let update = UserPlayOptionUpdateDto::new(1.25, 3);
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
