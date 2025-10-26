use domain::{
    entity::user::User,
    repository::{Repositories, user::UserRepository},
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
    use domain::entity::rating::Rating;
    use domain::{
        repository::{MockRepositories, record::MockRecordRepository, user::UserRepositoryError},
        testing::user::{USER1, USER2},
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn register_propagates_duplicate_card_error() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_create()
            .withf(|user| user.card() == USER1.card)
            .returning(|_| {
                Box::pin(async {
                    Err(UserRepositoryError::CardIdAlreadyExists(
                        USER1.card.to_owned(),
                    ))
                })
            });

        let repositories = MockRepositories {
            user: user_repo,
            record: MockRecordRepository::new(),
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let input = UserRegisterDto::new(USER1.card.to_owned(), USER1.display_name.to_owned());
        let result = usecase.register(input).await;

        match result {
            Err(UserUsecaseError::UserRepositoryError(
                UserRepositoryError::CardIdAlreadyExists(card),
            )) => assert_eq!(card, USER1.card),
            _ => panic!("unexpected result"),
        }
    }

    #[tokio::test]
    async fn register_returns_user_data_on_success() {
        let mut user_repo = domain::repository::user::MockUserRepository::new();
        user_repo
            .expect_create()
            .withf(|user| user.card() == USER2.card && user.display_name() == USER2.display_name)
            .returning(|user| {
                Box::pin(async move {
                    Ok(User::new(
                        USER1.id.to_owned(),
                        user.card().to_owned(),
                        user.display_name().clone(),
                        Rating::new(user.rating().value()),
                        *user.xp(),
                        *user.credits(),
                        *user.is_admin(),
                        *user.created_at(),
                    ))
                })
            });

        let repositories = MockRepositories {
            user: user_repo,
            record: MockRecordRepository::new(),
        };
        let usecase = UserUsecase::new(Arc::new(repositories));

        let input = UserRegisterDto::new(USER2.card.to_owned(), USER2.display_name.to_owned());
        let result = usecase.register(input).await.expect("should succeed");

        assert_eq!(result.card, USER2.card);
        assert_eq!(result.display_name, USER2.display_name);
        assert_eq!(result.id, USER1.id);
    }
}
