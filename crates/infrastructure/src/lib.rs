use domain::repository::Repositories;

pub mod entities;
pub mod model;
pub mod user;

pub struct RepositoriesImpl {
    user: user::UserRepositoryImpl,
}

impl RepositoriesImpl {
    pub fn new(user: user::UserRepositoryImpl) -> Self {
        Self { user }
    }

    pub async fn new_default(db_url: &str) -> Self {
        let db = sea_orm::Database::connect(db_url)
            .await
            .expect("Failed to connect to the database");

        let user_repo = user::UserRepositoryImpl::new(db);

        Self { user: user_repo }
    }
}

impl Repositories for RepositoriesImpl {
    type UserRepositoryImpl = user::UserRepositoryImpl;

    fn user(&self) -> &Self::UserRepositoryImpl {
        &self.user
    }
}
