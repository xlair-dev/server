use domain::{
    entity::user::User,
    repository::{user::UserRepository, Repositories},
};
use tracing::{debug, info, instrument};

use crate::{
    model::user::{UserDataDto, UserRegisterDto},
    user::{UserUsecase, UserUsecaseError},
};

impl<R: Repositories> UserUsecase<R> {
    #[instrument(skip(self, raw_user), fields(card = %raw_user.card))]
    pub async fn register(
        &self,
        raw_user: UserRegisterDto,
    ) -> Result<UserDataDto, UserUsecaseError> {
        let card = raw_user.card.clone();
        let display_name = raw_user.display_name.clone();
        debug!(%card, %display_name, "Building user aggregate");

        let user = User::new_temporary(raw_user.card, raw_user.display_name);
        let user = self.repositories.user().create(user).await?;
        info!(user_id = %user.id(), "User persisted by repository");
        Ok(user.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use domain::{
        entity::{rating::Rating, user::User},
        repository::{user::UserRepositoryError, MockRepositories},
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn register_propagates_duplicate_card_error() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_create()
            .withf(|user| user.card() == "CARD-001")
            .returning(|_| {
                Box::pin(async {
                    Err(UserRepositoryError::CardIdAlreadyExists(
                        "CARD-001".to_owned(),
                    ))
                })
            });

        let repositories = MockRepositories { user: user_repo };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let input = UserRegisterDto::new("CARD-001".to_owned(), "Alice".to_owned());
        let result = usecase.register(input).await;

        match result {
            Err(UserUsecaseError::UserRepositoryError(
                UserRepositoryError::CardIdAlreadyExists(card),
            )) => assert_eq!(card, "CARD-001"),
            _ => panic!("unexpected result"),
        }
    }

    #[tokio::test]
    async fn register_returns_user_data_on_success() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_create()
            .withf(|user| user.card() == "CARD-002" && user.display_name() == "Bob")
            .returning(|user| {
                Box::pin(async move {
                    let now: NaiveDateTime = *user.created_at();
                    let persisted = User::new(
                        "550e8400-e29b-41d4-a716-446655440000".to_owned(),
                        user.card().to_owned(),
                        user.display_name().clone(),
                        Rating::new(user.rating().value()),
                        *user.xp(),
                        *user.credits(),
                        *user.is_admin(),
                        now,
                    );
                    Ok(persisted)
                })
            });

        let repositories = MockRepositories { user: user_repo };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let input = UserRegisterDto::new("CARD-002".to_owned(), "Bob".to_owned());
        let result = usecase.register(input).await.expect("should succeed");

        assert_eq!(result.card, "CARD-002");
        assert_eq!(result.display_name, "Bob");
        assert_eq!(result.id, "550e8400-e29b-41d4-a716-446655440000");
    }
}
