mod adapter;
mod read;
mod write;

use domain::{
    entity::record::Record,
    repository::record::{RecordRepository, RecordRepositoryError, RecordWithMetadata},
};
use sea_orm::DbConn;
use std::sync::Arc;
use tracing::{debug, info, instrument};

use read::{records_by_user, records_by_user_and_sheet_ids, records_with_metadata_by_user};

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
        let records = records_by_user(self.db.as_ref(), user_id).await?;
        info!(count = records.len(), "Records fetched successfully");
        Ok(records)
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_with_metadata_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Vec<RecordWithMetadata>, RecordRepositoryError> {
        debug!("Fetching records with metadata via SeaORM");
        let result = records_with_metadata_by_user(self.db.as_ref(), user_id).await?;
        info!(
            count = result.len(),
            "Records with metadata fetched successfully"
        );
        Ok(result)
    }

    #[instrument(
        skip(self),
        fields(user_id = %user_id, sheet_count = sheet_ids.len())
    )]
    async fn find_by_user_id_and_sheet_ids(
        &self,
        user_id: &str,
        sheet_ids: &[String],
    ) -> Result<Vec<Record>, RecordRepositoryError> {
        debug!("Fetching records by sheet IDs via SeaORM");
        let records = records_by_user_and_sheet_ids(self.db.as_ref(), user_id, sheet_ids).await?;
        info!(
            count = records.len(),
            "Records by sheet IDs fetched successfully"
        );
        Ok(records)
    }

    #[instrument(skip(self, record), fields(user_id = %record.user_id(), sheet_id = %record.sheet_id()))]
    async fn insert(&self, record: Record) -> Result<Record, RecordRepositoryError> {
        debug!("Persisting record via SeaORM");
        let inserted = write::insert_record(self.db.as_ref(), record).await?;
        info!(record_id = %inserted.id(), "Record inserted successfully");
        Ok(inserted)
    }

    #[instrument(skip(self, record), fields(record_id = %record.id()))]
    async fn update(&self, record: Record) -> Result<Record, RecordRepositoryError> {
        debug!("Updating record via SeaORM");
        let updated = write::update_record(self.db.as_ref(), record).await?;
        info!(record_id = %updated.id(), "Record updated successfully");
        Ok(updated)
    }
}
