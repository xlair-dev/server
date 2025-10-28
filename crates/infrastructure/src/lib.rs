use domain::repository::Repositories;
use std::sync::Arc;
use tracing::{error, info, instrument};

pub mod entities;
pub mod model;
pub mod record;
pub mod user;

pub struct RepositoriesImpl {
    user: user::UserRepositoryImpl,
    record: record::RecordRepositoryImpl,
}

impl RepositoriesImpl {
    pub fn new(user: user::UserRepositoryImpl, record: record::RecordRepositoryImpl) -> Self {
        Self { user, record }
    }

    /// Initializes the SeaORM connection. Implicitly depends on a tracing subscriber already being set up so logging can emit.
    ///
    /// # Panics
    /// Panics if database connection fails. This is appropriate for startup as the application
    /// cannot function without a database connection.
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

        let db = Arc::new(db);
        let user_repo = user::UserRepositoryImpl::new(db.clone());
        let record_repo = record::RecordRepositoryImpl::new(db.clone());

        Self {
            user: user_repo,
            record: record_repo,
        }
    }
}

impl Repositories for RepositoriesImpl {
    type UserRepositoryImpl = user::UserRepositoryImpl;
    type RecordRepositoryImpl = record::RecordRepositoryImpl;

    fn user(&self) -> &Self::UserRepositoryImpl {
        &self.user
    }

    fn record(&self) -> &Self::RecordRepositoryImpl {
        &self.record
    }
}
