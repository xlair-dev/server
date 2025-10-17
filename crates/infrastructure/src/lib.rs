use domain::repository::Repositories;
use tracing::{error, info, instrument};

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

    /// Initializes the SeaORM connection. Implicitly depends on a tracing subscriber already being set up so logging can emit.
    #[instrument(name = "infrastructure.repositories.new_default", skip(db_url))]
    pub async fn new_default(db_url: &str) -> Self {
        info!("Connecting to database via SeaORM");
        let db = sea_orm::Database::connect(db_url)
            .await
            .unwrap_or_else(|err| {
                error!(error = %err, "Failed to connect to the database");
                panic!("Failed to connect to the database");
            });
        info!("Database connection established");

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
