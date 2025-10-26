use std::collections::HashMap;

use chrono::Utc;
use domain::{
    entity::{clear_type::ClearType, level::Level, record::Record},
    repository::{
        Repositories,
        record::{RecordRepository, RecordRepositoryError, RecordWithMetadata},
        user::UserRepository,
    },
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

        let existing_records = match self.repositories.record().find_by_user_id(&user_id).await {
            Ok(records) => records,
            Err(RecordRepositoryError::UserNotFound(_)) => {
                return Err(UserUsecaseError::NotFoundById { user_id });
            }
            Err(err) => return Err(UserUsecaseError::RecordRepositoryError(err)),
        };

        let mut record_map: HashMap<String, Record> = existing_records
            .into_iter()
            .map(|record| (record.sheet_id().to_owned(), record))
            .collect();

        let mut xp_delta: u32 = 0;
        let mut responses = Vec::with_capacity(submissions.len());

        for submission in submissions {
            let sheet_id = submission.sheet_id.clone();
            xp_delta = xp_delta.saturating_add(calculate_xp(submission.score));

            match record_map.remove(&sheet_id) {
                Some(mut record) => {
                    record.set_play_count(record.play_count() + 1);

                    if submission.score > *record.score() {
                        record.set_score(submission.score);
                    }

                    if is_better_clear(submission.clear_type, *record.clear_type()) {
                        record.set_clear_type(submission.clear_type);
                    }

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
                    let record = Record::new(
                        String::new(),
                        user_id.clone(),
                        sheet_id.clone(),
                        submission.score,
                        submission.clear_type,
                        1,
                        Utc::now().naive_utc(),
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

        let new_rating = calculate_rating(&metadata);

        if let Err(err) = self
            .repositories
            .user()
            .apply_progress(&user_id, xp_delta, new_rating)
            .await
        {
            return Err(match err {
                domain::repository::user::UserRepositoryError::NotFound(_) => {
                    UserUsecaseError::NotFoundById { user_id }
                }
                other => UserUsecaseError::UserRepositoryError(other),
            });
        }

        Ok(responses)
    }
}

fn calculate_xp(score: u32) -> u32 {
    let diff = score as i64 - 900_000;
    let bonus = diff / 1_000;
    if bonus < 1 { 1 } else { bonus as u32 }
}

fn is_better_clear(new: ClearType, current: ClearType) -> bool {
    clear_type_rank(new) > clear_type_rank(current)
}

fn clear_type_rank(clear_type: ClearType) -> u8 {
    match clear_type {
        ClearType::Fail => 0,
        ClearType::Clear => 1,
        ClearType::FullCombo => 2,
        ClearType::AllPerfect => 3,
    }
}

fn calculate_rating(records: &[RecordWithMetadata]) -> u32 {
    let mut ratings: Vec<u32> = records
        .iter()
        .filter(|entry| !entry.is_test)
        .map(|entry| calculate_single_track_rating(&entry.level, *entry.record.score()))
        .collect();

    if ratings.is_empty() {
        return 0;
    }

    ratings.sort_unstable_by(|a, b| b.cmp(a));
    let count = ratings.len().min(3);
    let total: u32 = ratings.into_iter().take(count).sum();
    total / (count as u32)
}

fn calculate_single_track_rating(level: &Level, score: u32) -> u32 {
    let (integer, decimal) = level_components(level);
    let base = integer * 100 + decimal * 10;
    let bonus = compute_score_bonus(score);
    let total = base as i64 + bonus as i64;
    if total < 0 { 0 } else { total as u32 }
}

fn level_components(level: &Level) -> (u32, u32) {
    level.components()
}

fn compute_score_bonus(score: u32) -> i32 {
    const ANCHORS: [(u32, i32); 9] = [
        (700_000, -200),
        (750_000, -150),
        (800_000, -100),
        (850_000, -50),
        (900_000, 0),
        (950_000, 50),
        (1_000_000, 100),
        (1_050_000, 150),
        (1_090_000, 200),
    ];

    if score <= ANCHORS[0].0 {
        return ANCHORS[0].1;
    }

    if score >= ANCHORS[ANCHORS.len() - 1].0 {
        return ANCHORS[ANCHORS.len() - 1].1;
    }

    for window in ANCHORS.windows(2) {
        let lower = window[0];
        let upper = window[1];

        if (lower.0..=upper.0).contains(&score) {
            let range = (upper.0 - lower.0) as i64;
            let position = (score - lower.0) as i64;
            let diff = (upper.1 - lower.1) as i64;
            let bonus = lower.1 as i64 + diff * position / range;
            return bonus as i32;
        }
    }

    ANCHORS[0].1
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use domain::{
        entity::{clear_type::ClearType, level::Level, record::Record},
        repository::{
            MockRepositories,
            record::{MockRecordRepository, RecordRepositoryError, RecordWithMetadata},
            user::{MockUserRepository, UserRepositoryError},
        },
    };
    use std::sync::Arc;

    fn sample_timestamp() -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(2025, 10, 26)
            .unwrap()
            .and_hms_opt(9, 30, 0)
            .unwrap()
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
            .expect_find_by_user_id()
            .withf(|user_id| user_id == "user-123")
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
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
            .expect_apply_progress()
            .withf(|user_id, xp_delta, rating| {
                user_id == "user-123" && *xp_delta == 100 && *rating == 1470
            })
            .returning(|_, _, _| Box::pin(async { Ok(()) }));

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
            .expect_find_by_user_id()
            .withf(|user_id| user_id == "user-456")
            .returning(|_| {
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
            .expect_apply_progress()
            .withf(|user_id, xp_delta, rating| {
                user_id == "user-456" && *xp_delta == 80 && *rating == 1330
            })
            .returning(|_, _, _| Box::pin(async { Ok(()) }));

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
        record_repo.expect_find_by_user_id().returning(|_| {
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
            .expect_find_by_user_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
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
        user_repo.expect_apply_progress().returning(|_, _, _| {
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
