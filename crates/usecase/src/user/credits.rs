use domain::repository::{Repositories, user::UserRepository};
use tracing::{debug, instrument};

use crate::{
    model::user::UserCreditsDto,
    user::{UserUsecase, UserUsecaseError},
};

impl<R: Repositories> UserUsecase<R> {
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn increment_credits(
        &self,
        user_id: String,
    ) -> Result<UserCreditsDto, UserUsecaseError> {
        debug!("Incrementing credits via usecase");
        match self.repositories.user().increment_credits(&user_id).await {
            Ok(credits) => Ok(UserCreditsDto::new(credits)),
            Err(domain::repository::user::UserRepositoryError::NotFound(_)) => {
                Err(UserUsecaseError::NotFoundById { user_id })
            }
            Err(err) => Err(UserUsecaseError::UserRepositoryError(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use domain::repository::{
        MockRepositories,
        user::{MockUserRepository, UserRepositoryError},
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn increment_credits_returns_new_value() {
        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_increment_credits()
            .withf(|user_id| user_id == "user-123")
            .returning(|_| Box::pin(async { Ok(12) }));

        let repositories = MockRepositories { user: user_repo };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let response = usecase
            .increment_credits("user-123".to_owned())
            .await
            .expect("should succeed");

        assert_eq!(response.credits, 12);
    }

    #[tokio::test]
    async fn increment_credits_maps_not_found() {
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_increment_credits().returning(|_| {
            Box::pin(async { Err(UserRepositoryError::NotFound("user-404".to_owned())) })
        });

        let repositories = MockRepositories { user: user_repo };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let err = usecase
            .increment_credits("user-404".to_owned())
            .await
            .expect_err("should map to not found");

        match err {
            UserUsecaseError::NotFoundById { user_id } => assert_eq!(user_id, "user-404"),
            _ => panic!("unexpected error variant"),
        }
    }

    #[tokio::test]
    async fn increment_credits_propagates_other_errors() {
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_increment_credits().returning(|_| {
            Box::pin(async { Err(UserRepositoryError::InternalError(anyhow!("boom"))) })
        });

        let repositories = MockRepositories { user: user_repo };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let err = usecase
            .increment_credits("user-err".to_owned())
            .await
            .expect_err("should propagate repository error");

        match err {
            UserUsecaseError::UserRepositoryError(UserRepositoryError::InternalError(inner)) => {
                assert_eq!(inner.to_string(), "boom");
            }
            _ => panic!("unexpected error variant"),
        }
    }
}
