use std::sync::Arc;

use domain::repository::{
    Repositories,
    record::{RecordRepository, RecordRepositoryError, SheetScoreRankingRow, TotalScoreRankingRow},
    user::{UserRepository, UserRepositoryError},
};
use thiserror::Error;

use crate::model::ranking::{
    RatingRankingDto, RatingRankingEntryDto, SheetScoreRankingDto, SheetScoreRankingEntryDto,
    TotalScoreRankingDto, TotalScoreRankingEntryDto, XpRankingDto, XpRankingEntryDto,
};

const DEFAULT_LIMIT: u64 = 20;

#[derive(Debug, Error)]
pub enum RankingUsecaseError {
    #[error(transparent)]
    RecordRepository(#[from] RecordRepositoryError),
    #[error(transparent)]
    UserRepository(#[from] UserRepositoryError),
}

pub struct RankingUsecase<R: Repositories> {
    repositories: Arc<R>,
    limit: u64,
}

impl<R: Repositories> RankingUsecase<R> {
    /// Instantiates the usecase with the default ranking size (20 entries).
    pub fn new(repositories: Arc<R>) -> Self {
        Self {
            repositories,
            limit: DEFAULT_LIMIT,
        }
    }

    fn limit(&self) -> u64 {
        self.limit
    }

    pub async fn sheet_high_scores(
        &self,
        sheet_id: &str,
    ) -> Result<SheetScoreRankingDto, RankingUsecaseError> {
        let rows = self
            .repositories
            .record()
            .find_public_high_scores_by_sheet(sheet_id, self.limit())
            .await?;

        Ok(SheetScoreRankingDto::new(
            sheet_id.to_owned(),
            self.decorate_sheet_rows(rows),
        ))
    }

    pub async fn total_high_scores(&self) -> Result<TotalScoreRankingDto, RankingUsecaseError> {
        let rows = self
            .repositories
            .record()
            .find_public_total_score_ranking(self.limit())
            .await?;
        Ok(TotalScoreRankingDto::new(self.decorate_total_rows(rows)))
    }

    pub async fn rating(&self) -> Result<RatingRankingDto, RankingUsecaseError> {
        let users = self
            .repositories
            .user()
            .find_public_top_by_rating(self.limit())
            .await?;
        Ok(RatingRankingDto::new(
            users
                .into_iter()
                .enumerate()
                .map(|(idx, user)| {
                    RatingRankingEntryDto::new(
                        (idx as u32) + 1,
                        user.id().to_owned(),
                        user.display_name().clone(),
                        user.rating().value(),
                    )
                })
                .collect(),
        ))
    }

    pub async fn xp(&self) -> Result<XpRankingDto, RankingUsecaseError> {
        let users = self
            .repositories
            .user()
            .find_public_top_by_xp(self.limit())
            .await?;
        Ok(XpRankingDto::new(
            users
                .into_iter()
                .enumerate()
                .map(|(idx, user)| {
                    XpRankingEntryDto::new(
                        (idx as u32) + 1,
                        user.id().to_owned(),
                        user.display_name().clone(),
                        *user.xp(),
                    )
                })
                .collect(),
        ))
    }

    fn decorate_sheet_rows(
        &self,
        rows: Vec<SheetScoreRankingRow>,
    ) -> Vec<SheetScoreRankingEntryDto> {
        rows.into_iter()
            .enumerate()
            .map(|(idx, row)| {
                SheetScoreRankingEntryDto::new(
                    (idx as u32) + 1,
                    row.user_id,
                    row.display_name,
                    row.score,
                )
            })
            .collect()
    }

    fn decorate_total_rows(
        &self,
        rows: Vec<TotalScoreRankingRow>,
    ) -> Vec<TotalScoreRankingEntryDto> {
        rows.into_iter()
            .enumerate()
            .map(|(idx, row)| {
                TotalScoreRankingEntryDto::new(
                    (idx as u32) + 1,
                    row.user_id,
                    row.display_name,
                    row.total_score,
                )
            })
            .collect()
    }
}

impl<R: Repositories> Clone for RankingUsecase<R> {
    fn clone(&self) -> Self {
        Self {
            repositories: Arc::clone(&self.repositories),
            limit: self.limit,
        }
    }
}
