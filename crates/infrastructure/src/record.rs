use anyhow::Error as AnyError;
use domain::{
    entity::record::Record,
    repository::record::{RecordRepository, RecordRepositoryError},
};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, prelude::Uuid};
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

use crate::{entities, entities::prelude::Records};

pub struct RecordRepositoryImpl {
    db: Arc<DbConn>,
}

impl RecordRepositoryImpl {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }
}

impl RecordRepository for RecordRepositoryImpl {
    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Record>, RecordRepositoryError> {
        debug!("Fetching records via SeaORM");

        let uuid = Uuid::parse_str(user_id).map_err(|err| {
            debug!(error = %err, "Failed to parse user id");
            RecordRepositoryError::UserNotFound(user_id.to_owned())
        })?;

        let user_exists = entities::users::Entity::find_by_id(uuid)
            .one(self.db.as_ref())
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to verify user existence");
                RecordRepositoryError::InternalError(AnyError::from(err))
            })?;

        if user_exists.is_none() {
            debug!("User not found while querying records");
            return Err(RecordRepositoryError::UserNotFound(user_id.to_owned()));
        }

        let models = Records::find()
            .filter(entities::records::Column::UserId.eq(uuid))
            .all(self.db.as_ref())
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to fetch records");
                RecordRepositoryError::InternalError(AnyError::from(err))
            })?;

        info!(count = models.len(), "Records fetched successfully");
        Ok(models.into_iter().map(Record::from).collect())
    }
}
