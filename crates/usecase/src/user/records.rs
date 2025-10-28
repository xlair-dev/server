use std::collections::HashMap;

use chrono::Utc;
use domain::{
    entity::record::Record,
    repository::{
        Repositories,
        record::{RecordRepository, RecordRepositoryError},
        user::UserRepository,
    },
    service::{experience, rating},
};
use tracing::{debug, instrument};

use crate::{
    model::user::{UserRecordDto, UserRecordSubmissionDto},
    user::{UserUsecase, UserUsecaseError},
};

impl<R: Repositories> UserUsecase<R> {
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn list_records(
        &self,
        user_id: String,
    ) -> Result<Vec<UserRecordDto>, UserUsecaseError> {
        debug!("Resolving records for user");
        match self.repositories.record().find_by_user_id(&user_id).await {
            Ok(records) => {
                let result = records.into_iter().map(UserRecordDto::from).collect();
                Ok(result)
            }
            Err(RecordRepositoryError::UserNotFound(_)) => {
                Err(UserUsecaseError::NotFoundById { user_id })
            }
            Err(err) => Err(UserUsecaseError::RecordRepositoryError(err)),
        }
    }

    #[instrument(skip(self, submissions), fields(user_id = %user_id, count = submissions.len()))]
    pub async fn submit_records(
        &self,
        user_id: String,
        submissions: Vec<UserRecordSubmissionDto>,
    ) -> Result<Vec<UserRecordDto>, UserUsecaseError> {
        debug!("Processing record submissions");

        if submissions.is_empty() {
            debug!("No submissions provided");
            return Ok(Vec::new());
        }

        let sheet_ids: Vec<String> = submissions.iter().map(|s| s.sheet_id.clone()).collect();
        let existing_records = match self
            .repositories
            .record()
            .find_by_user_id_and_sheet_ids(&user_id, &sheet_ids)
            .await
        {
            Ok(records) => records,
            Err(RecordRepositoryError::UserNotFound(_)) => {
                return Err(UserUsecaseError::NotFoundById { user_id });
            }
            Err(err) => return Err(UserUsecaseError::RecordRepositoryError(err)),
        };

        let mut user = self
            .repositories
            .user()
            .find_by_id(&user_id)
            .await?
            .ok_or_else(|| UserUsecaseError::NotFoundById {
                user_id: user_id.clone(),
            })?;

        let mut record_map: HashMap<String, Record> = existing_records
            .into_iter()
            .map(|record| (record.sheet_id().to_owned(), record))
            .collect();

        let mut xp_delta: u32 = 0;
        let mut responses = Vec::with_capacity(submissions.len());

        for submission in submissions {
            let sheet_id = submission.sheet_id.clone();
            xp_delta = xp_delta.saturating_add(experience::xp_for_score(submission.score));
            let submitted_at = Utc::now();

            match record_map.remove(&sheet_id) {
                Some(mut record) => {
                    record.apply_submission(submission.score, submission.clear_type, submitted_at);

                    let updated = match self.repositories.record().update(record).await {
                        Ok(value) => value,
                        Err(RecordRepositoryError::UserNotFound(_)) => {
                            return Err(UserUsecaseError::NotFoundById {
                                user_id: user_id.clone(),
                            });
                        }
                        Err(err) => return Err(UserUsecaseError::RecordRepositoryError(err)),
                    };

                    record_map.insert(sheet_id, updated.clone());
                    responses.push(UserRecordDto::from(updated));
                }
                None => {
                    let record = Record::new_from_submission(
                        user_id.clone(),
                        sheet_id.clone(),
                        submission.score,
                        submission.clear_type,
                        submitted_at,
                    );

                    let inserted = match self.repositories.record().insert(record).await {
                        Ok(value) => value,
                        Err(RecordRepositoryError::UserNotFound(_)) => {
                            return Err(UserUsecaseError::NotFoundById {
                                user_id: user_id.clone(),
                            });
                        }
                        Err(err) => return Err(UserUsecaseError::RecordRepositoryError(err)),
                    };

                    record_map.insert(sheet_id, inserted.clone());
                    responses.push(UserRecordDto::from(inserted));
                }
            }
        }

        let metadata = match self
            .repositories
            .record()
            .find_with_metadata_by_user_id(&user_id)
            .await
        {
            Ok(data) => data,
            Err(RecordRepositoryError::UserNotFound(_)) => {
                return Err(UserUsecaseError::NotFoundById { user_id });
            }
            Err(err) => return Err(UserUsecaseError::RecordRepositoryError(err)),
        };

        let new_rating = rating::calculate_user_rating(&metadata);

        user.add_xp(xp_delta);
        user.update_rating(new_rating);

        self.repositories.user().save(user).await?;

        Ok(responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        entity::rating::Rating,
        entity::{clear_type::ClearType, level::Level, record::Record, user::User},
        repository::{
            MockRepositories,
            record::{MockRecordRepository, RecordRepositoryError, RecordWithMetadata},
            user::{MockUserRepository, UserRepositoryError},
        },
    };
    use std::sync::Arc;

    fn sample_timestamp() -> chrono::DateTime<chrono::Utc> {
        chrono::NaiveDate::from_ymd_opt(2025, 10, 26)
            .unwrap()
            .and_hms_opt(9, 30, 0)
            .unwrap()
            .and_utc()
    }

    #[tokio::test]
    async fn list_records_returns_records() {
        let mut record_repo = MockRecordRepository::new();
        record_repo
            .expect_find_by_user_id()
            .withf(|user_id| user_id == "user-123")
            .returning(|_| {
                Box::pin(async move {
                    Ok(vec![Record::new(
                        "record-1".to_owned(),
                        "user-123".to_owned(),
                        "sheet-1".to_owned(),
                        1_050_000,
                        ClearType::FullCombo,
                        7,
                        sample_timestamp(),
                    )])
                })
            });

        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: record_repo,
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let result = usecase
            .list_records("user-123".to_owned())
            .await
            .expect("should succeed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "record-1");
        assert_eq!(result[0].sheet_id, "sheet-1");
        assert_eq!(result[0].clear_type, ClearType::FullCombo);
    }

    #[tokio::test]
    async fn list_records_maps_user_not_found() {
        let mut record_repo = MockRecordRepository::new();
        record_repo.expect_find_by_user_id().returning(|_| {
            Box::pin(async { Err(RecordRepositoryError::UserNotFound("missing".to_owned())) })
        });

        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: record_repo,
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let err = usecase
            .list_records("missing".to_owned())
            .await
            .expect_err("should return not found");

        match err {
            UserUsecaseError::NotFoundById { user_id } => assert_eq!(user_id, "missing"),
            _ => panic!("unexpected error variant"),
        }
    }

    #[tokio::test]
    async fn list_records_wraps_other_errors() {
        let mut record_repo = MockRecordRepository::new();
        record_repo.expect_find_by_user_id().returning(|_| {
            Box::pin(async {
                Err(RecordRepositoryError::InternalError(anyhow::anyhow!(
                    "db error"
                )))
            })
        });

        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: record_repo,
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let err = usecase
            .list_records("user-err".to_owned())
            .await
            .expect_err("should propagate repo error");

        match err {
            UserUsecaseError::RecordRepositoryError(RecordRepositoryError::InternalError(
                inner,
            )) => {
                assert_eq!(inner.to_string(), "db error");
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[tokio::test]
    async fn submit_records_creates_new_entries() {
        let mut record_repo = MockRecordRepository::new();
        record_repo
            .expect_find_by_user_id_and_sheet_ids()
            .withf(|user_id, sheet_ids| {
                user_id == "user-123" && sheet_ids.len() == 1 && sheet_ids[0] == "sheet-1"
            })
            .returning(|_, _| Box::pin(async { Ok(Vec::new()) }));
        record_repo
            .expect_insert()
            .withf(|record| record.user_id() == "user-123" && record.sheet_id() == "sheet-1")
            .returning(|record| {
                Box::pin(async move {
                    Ok(Record::new(
                        record.id().to_owned(),
                        record.user_id().to_owned(),
                        record.sheet_id().to_owned(),
                        *record.score(),
                        *record.clear_type(),
                        *record.play_count(),
                        sample_timestamp(),
                    ))
                })
            });
        record_repo
            .expect_find_with_metadata_by_user_id()
            .withf(|user_id| user_id == "user-123")
            .returning(|_| {
                Box::pin(async move {
                    let level = Level::new(13, 7).expect("level should be valid");
                    let record = Record::new(
                        "record-1".to_owned(),
                        "user-123".to_owned(),
                        "sheet-1".to_owned(),
                        1_000_000,
                        ClearType::FullCombo,
                        1,
                        sample_timestamp(),
                    );
                    Ok(vec![RecordWithMetadata::new(record, level, false)])
                })
            });

        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_by_id()
            .withf(|user_id| user_id == "user-123")
            .returning(|_| {
                let user = User::new(
                    "user-123".to_owned(),
                    "CARD-123".to_owned(),
                    "Alice".to_owned(),
                    Rating::new(1200),
                    0,
                    0,
                    false,
                    sample_timestamp(),
                );
                Box::pin(async move { Ok(Some(user)) })
            });
        user_repo
            .expect_save()
            .withf(|user| {
                user.id() == "user-123" && *user.xp() == 100 && user.rating().value() == 1470
            })
            .returning(|user| Box::pin(async move { Ok(user) }));

        let repositories = MockRepositories {
            user: user_repo,
            record: record_repo,
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let submission =
            UserRecordSubmissionDto::new("sheet-1".to_owned(), 1_000_000, ClearType::FullCombo);
        let result = usecase
            .submit_records("user-123".to_owned(), vec![submission])
            .await
            .expect("should succeed");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].sheet_id, "sheet-1");
        assert_eq!(result[0].score, 1_000_000);
    }

    #[tokio::test]
    async fn submit_records_updates_existing_entries() {
        let mut record_repo = MockRecordRepository::new();
        record_repo
            .expect_find_by_user_id_and_sheet_ids()
            .withf(|user_id, sheet_ids| {
                user_id == "user-456" && sheet_ids.len() == 1 && sheet_ids[0] == "sheet-1"
            })
            .returning(|_, _| {
                Box::pin(async move {
                    Ok(vec![Record::new(
                        "record-1".to_owned(),
                        "user-456".to_owned(),
                        "sheet-1".to_owned(),
                        950_000,
                        ClearType::Clear,
                        3,
                        sample_timestamp(),
                    )])
                })
            });
        record_repo
            .expect_update()
            .withf(|record| {
                record.user_id() == "user-456"
                    && record.sheet_id() == "sheet-1"
                    && *record.score() == 980_000
                    && record.play_count() == &4
            })
            .returning(|record| {
                Box::pin(async move {
                    Ok(Record::new(
                        record.id().to_owned(),
                        record.user_id().to_owned(),
                        record.sheet_id().to_owned(),
                        *record.score(),
                        *record.clear_type(),
                        *record.play_count(),
                        sample_timestamp(),
                    ))
                })
            });
        record_repo
            .expect_find_with_metadata_by_user_id()
            .withf(|user_id| user_id == "user-456")
            .returning(|_| {
                Box::pin(async move {
                    let level = Level::new(12, 5).expect("level should be valid");
                    let record = Record::new(
                        "record-1".to_owned(),
                        "user-456".to_owned(),
                        "sheet-1".to_owned(),
                        980_000,
                        ClearType::FullCombo,
                        4,
                        sample_timestamp(),
                    );
                    Ok(vec![RecordWithMetadata::new(record, level, false)])
                })
            });

        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_by_id()
            .withf(|user_id| user_id == "user-456")
            .returning(|_| {
                let user = User::new(
                    "user-456".to_owned(),
                    "CARD-456".to_owned(),
                    "Bob".to_owned(),
                    Rating::new(1200),
                    100,
                    0,
                    false,
                    sample_timestamp(),
                );
                Box::pin(async move { Ok(Some(user)) })
            });
        user_repo
            .expect_save()
            .withf(|user| {
                user.id() == "user-456" && *user.xp() == 180 && user.rating().value() == 1330
            })
            .returning(|user| Box::pin(async move { Ok(user) }));

        let repositories = MockRepositories {
            user: user_repo,
            record: record_repo,
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let submission =
            UserRecordSubmissionDto::new("sheet-1".to_owned(), 980_000, ClearType::FullCombo);
        let result = usecase
            .submit_records("user-456".to_owned(), vec![submission])
            .await
            .expect("should succeed");

        assert_eq!(result[0].score, 980_000);
        assert_eq!(result[0].clear_type, ClearType::FullCombo);
        assert_eq!(result[0].play_count, 4);
    }

    #[tokio::test]
    async fn submit_records_maps_user_not_found() {
        let mut record_repo = MockRecordRepository::new();
        record_repo
            .expect_find_by_user_id_and_sheet_ids()
            .returning(|_, _| {
                Box::pin(async { Err(RecordRepositoryError::UserNotFound("missing".to_owned())) })
            });

        let repositories = MockRepositories {
            user: MockUserRepository::new(),
            record: record_repo,
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let submission =
            UserRecordSubmissionDto::new("sheet-1".to_owned(), 900_000, ClearType::Clear);

        let err = usecase
            .submit_records("missing".to_owned(), vec![submission])
            .await
            .expect_err("should return not found");

        match err {
            UserUsecaseError::NotFoundById { user_id } => assert_eq!(user_id, "missing"),
            _ => panic!("unexpected error variant"),
        }
    }

    #[tokio::test]
    async fn submit_records_propagates_user_repo_errors() {
        let mut record_repo = MockRecordRepository::new();
        record_repo
            .expect_find_by_user_id_and_sheet_ids()
            .returning(|_, _| Box::pin(async { Ok(Vec::new()) }));
        record_repo.expect_insert().returning(|record| {
            Box::pin(async move {
                Ok(Record::new(
                    record.id().to_owned(),
                    record.user_id().to_owned(),
                    record.sheet_id().to_owned(),
                    *record.score(),
                    *record.clear_type(),
                    *record.play_count(),
                    sample_timestamp(),
                ))
            })
        });
        record_repo
            .expect_find_with_metadata_by_user_id()
            .returning(|_| {
                Box::pin(async move {
                    let level = Level::new(11, 0).expect("level valid");
                    let record = Record::new(
                        "record-1".to_owned(),
                        "user-789".to_owned(),
                        "sheet-1".to_owned(),
                        910_000,
                        ClearType::Clear,
                        1,
                        sample_timestamp(),
                    );
                    Ok(vec![RecordWithMetadata::new(record, level, false)])
                })
            });

        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_by_id()
            .withf(|user_id| user_id == "user-789")
            .returning(|_| {
                let user = User::new(
                    "user-789".to_owned(),
                    "CARD-789".to_owned(),
                    "Charlie".to_owned(),
                    Rating::new(1100),
                    50,
                    0,
                    false,
                    sample_timestamp(),
                );
                Box::pin(async move { Ok(Some(user)) })
            });
        user_repo.expect_save().returning(|_| {
            Box::pin(async { Err(UserRepositoryError::InternalError(anyhow::anyhow!("boom"))) })
        });

        let repositories = MockRepositories {
            user: user_repo,
            record: record_repo,
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let submission =
            UserRecordSubmissionDto::new("sheet-1".to_owned(), 910_000, ClearType::Clear);

        let err = usecase
            .submit_records("user-789".to_owned(), vec![submission])
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
