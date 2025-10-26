use domain::repository::{Repositories, record::RecordRepository};
use tracing::{debug, instrument};

use crate::{
    model::user::UserRecordDto,
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
            Err(domain::repository::record::RecordRepositoryError::UserNotFound(_)) => {
                Err(UserUsecaseError::NotFoundById { user_id })
            }
            Err(err) => Err(UserUsecaseError::RecordRepositoryError(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use domain::{
        entity::{clear_type::ClearType, record::Record},
        repository::{
            MockRepositories,
            record::{MockRecordRepository, RecordRepositoryError},
            user::MockUserRepository,
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
}
