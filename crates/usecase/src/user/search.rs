use domain::repository::{Repositories, user::UserRepository};
use tracing::{debug, instrument};

use crate::{
    model::user::UserDataDto,
    user::{UserUsecase, UserUsecaseError},
};

impl<R: Repositories> UserUsecase<R> {
    #[instrument(skip(self), fields(card = %card))]
    pub async fn find_by_card(&self, card: String) -> Result<UserDataDto, UserUsecaseError> {
        debug!(%card, "Resolving user aggregate by card");
        let maybe_user = self.repositories.user().find_by_card(&card).await?;
        let user =
            maybe_user.ok_or_else(|| UserUsecaseError::NotFoundByCard { card: card.clone() })?;
        debug!(user_id = %user.id(), "User aggregate resolved");
        Ok(user.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::repository::{
        MockRepositories, record::MockRecordRepository, user::UserRepositoryError,
    };
    use domain::testing::{
        datetime::sample_timestamp,
        user::{USER1, USER2},
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn find_by_card_returns_user_when_found() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_find_by_card()
            .withf(|card| card == USER1.card)
            .returning(|_| {
                let aggregate = USER1.build(sample_timestamp(), false);
                Box::pin(async move { Ok(Some(aggregate)) })
            });

        let repositories = MockRepositories {
            user: user_repo,
            record: MockRecordRepository::new(),
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let result = usecase
            .find_by_card(USER1.card.to_owned())
            .await
            .expect("should succeed");

        assert_eq!(result.id, USER1.id);
        assert_eq!(result.card, USER1.card);
    }

    #[tokio::test]
    async fn find_by_card_returns_error_when_repository_returns_none() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_find_by_card()
            .withf(|card| card == USER2.card)
            .returning(|_| Box::pin(async { Ok(None) }));

        let repositories = MockRepositories {
            user: user_repo,
            record: MockRecordRepository::new(),
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let err = usecase
            .find_by_card(USER2.card.to_owned())
            .await
            .expect_err("should return not found error");

        match err {
            UserUsecaseError::NotFoundByCard { card } => assert_eq!(card, USER2.card),
            _ => panic!("unexpected error variant"),
        }
    }

    #[tokio::test]
    async fn find_by_card_propagates_repository_errors() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo.expect_find_by_card().returning(|_| {
            Box::pin(async {
                Err(UserRepositoryError::InternalError(anyhow::anyhow!(
                    "db error"
                )))
            })
        });

        let repositories = MockRepositories {
            user: user_repo,
            record: MockRecordRepository::new(),
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let err = usecase
            .find_by_card(USER1.card.to_owned())
            .await
            .expect_err("should propagate repository errors");

        match err {
            UserUsecaseError::UserRepositoryError(UserRepositoryError::InternalError(inner)) => {
                assert_eq!(inner.to_string(), "db error");
            }
            _ => panic!("unexpected error variant"),
        }
    }
}
