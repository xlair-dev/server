mod mapper;
mod query;

use anyhow::Error as AnyError;
use domain::{
    entity::record::Record,
    repository::record::{RecordRepository, RecordRepositoryError, RecordWithMetadata},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, prelude::Uuid};
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

use crate::entities::{self, prelude::Records};
use mapper::{
    active_model_for_insert, active_model_for_update, convert_insert_error, convert_update_error,
};
use query::{ensure_user_exists, records_with_metadata};

pub struct RecordRepositoryImpl {
    db: Arc<DbConn>,
}

impl RecordRepositoryImpl {
    pub fn new(db: Arc<DbConn>) -> Self {
        Self { db }
    }

    async fn resolve_user_uuid(&self, user_id: &str) -> Result<Uuid, RecordRepositoryError> {
        ensure_user_exists(self.db.as_ref(), user_id).await
    }
}

impl RecordRepository for RecordRepositoryImpl {
    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Record>, RecordRepositoryError> {
        debug!("Fetching records via SeaORM");

        let uuid = self.resolve_user_uuid(user_id).await?;

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

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_with_metadata_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Vec<RecordWithMetadata>, RecordRepositoryError> {
        debug!("Fetching records with metadata via SeaORM");

        let uuid = self.resolve_user_uuid(user_id).await?;

        let result = records_with_metadata(self.db.as_ref(), uuid).await?;

        info!(
            count = result.len(),
            "Records with metadata fetched successfully"
        );
        Ok(result)
    }

    #[instrument(skip(self, record), fields(user_id = %record.user_id(), sheet_id = %record.sheet_id()))]
    async fn insert(&self, record: Record) -> Result<Record, RecordRepositoryError> {
        debug!("Persisting record via SeaORM");

        let user_id = record.user_id().to_owned();
        let sheet_id = record.sheet_id().to_owned();
        let active = active_model_for_insert(&record)?;

        match active.insert(self.db.as_ref()).await {
            Ok(model) => {
                info!(record_id = %model.id, "Record inserted successfully");
                Ok(model.into())
            }
            Err(err) => {
                error!(error = %err, "Failed to insert record");
                Err(convert_insert_error(err, &user_id, &sheet_id))
            }
        }
    }

    #[instrument(skip(self, record), fields(record_id = %record.id()))]
    async fn update(&self, record: Record) -> Result<Record, RecordRepositoryError> {
        debug!("Updating record via SeaORM");

        let record_id = record.id().to_owned();
        let active = active_model_for_update(&record)?;

        match active.update(self.db.as_ref()).await {
            Ok(model) => {
                info!(record_id = %model.id, "Record updated successfully");
                Ok(model.into())
            }
            Err(err) => {
                error!(error = %err, "Failed to update record");
                Err(convert_update_error(err, &record_id))
            }
        }
    }
}
