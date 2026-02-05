use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::TeamSeason;
use domain::repositories::TeamSeasonRepository;

use crate::errors::DbError;
use crate::models::TeamSeasonDb;

/// SQLx implementation of TeamSeasonRepository
pub struct SqlxTeamSeasonRepository {
    pool: PgPool,
}

impl SqlxTeamSeasonRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TeamSeasonRepository for SqlxTeamSeasonRepository {
    async fn create(&self, season: &TeamSeason) -> DomainResult<TeamSeason> {
        let season_db = TeamSeasonDb::from_domain(season);

        let result = sqlx::query_as!(
            TeamSeasonDb,
            r#"
            INSERT INTO team_seasons (id, team_id, season_year, wins, losses, ties, playoff_result, draft_position, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, team_id, season_year, wins, losses, ties, playoff_result, draft_position, created_at, updated_at
            "#,
            season_db.id,
            season_db.team_id,
            season_db.season_year,
            season_db.wins,
            season_db.losses,
            season_db.ties,
            season_db.playoff_result,
            season_db.draft_position,
            season_db.created_at,
            season_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(format!(
                        "Team season for team {} in year {} already exists",
                        season_db.team_id, season_db.season_year
                    ));
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn upsert(&self, season: &TeamSeason) -> DomainResult<TeamSeason> {
        let season_db = TeamSeasonDb::from_domain(season);

        let result = sqlx::query_as!(
            TeamSeasonDb,
            r#"
            INSERT INTO team_seasons (id, team_id, season_year, wins, losses, ties, playoff_result, draft_position, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (team_id, season_year) DO UPDATE SET
                wins = EXCLUDED.wins,
                losses = EXCLUDED.losses,
                ties = EXCLUDED.ties,
                playoff_result = EXCLUDED.playoff_result,
                draft_position = EXCLUDED.draft_position,
                updated_at = NOW()
            RETURNING id, team_id, season_year, wins, losses, ties, playoff_result, draft_position, created_at, updated_at
            "#,
            season_db.id,
            season_db.team_id,
            season_db.season_year,
            season_db.wins,
            season_db.losses,
            season_db.ties,
            season_db.playoff_result,
            season_db.draft_position,
            season_db.created_at,
            season_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<TeamSeason>> {
        let result = sqlx::query_as!(
            TeamSeasonDb,
            r#"
            SELECT id, team_id, season_year, wins, losses, ties, playoff_result, draft_position, created_at, updated_at
            FROM team_seasons
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(season_db) => Ok(Some(season_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_team_and_year(
        &self,
        team_id: Uuid,
        year: i32,
    ) -> DomainResult<Option<TeamSeason>> {
        let result = sqlx::query_as!(
            TeamSeasonDb,
            r#"
            SELECT id, team_id, season_year, wins, losses, ties, playoff_result, draft_position, created_at, updated_at
            FROM team_seasons
            WHERE team_id = $1 AND season_year = $2
            "#,
            team_id,
            year
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(season_db) => Ok(Some(season_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_year(&self, year: i32) -> DomainResult<Vec<TeamSeason>> {
        let results = sqlx::query_as!(
            TeamSeasonDb,
            r#"
            SELECT id, team_id, season_year, wins, losses, ties, playoff_result, draft_position, created_at, updated_at
            FROM team_seasons
            WHERE season_year = $1
            ORDER BY draft_position ASC NULLS LAST, wins DESC, losses ASC
            "#,
            year
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|season_db| season_db.to_domain().map_err(Into::into))
            .collect()
    }

    async fn find_by_year_ordered_by_draft_position(
        &self,
        year: i32,
    ) -> DomainResult<Vec<TeamSeason>> {
        let results = sqlx::query_as!(
            TeamSeasonDb,
            r#"
            SELECT id, team_id, season_year, wins, losses, ties, playoff_result, draft_position, created_at, updated_at
            FROM team_seasons
            WHERE season_year = $1 AND draft_position IS NOT NULL
            ORDER BY draft_position ASC
            "#,
            year
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|season_db| season_db.to_domain().map_err(Into::into))
            .collect()
    }

    async fn delete_by_year(&self, year: i32) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM team_seasons
            WHERE season_year = $1
            "#,
            year
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM team_seasons
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("Team season with id {} not found", id)).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_pool;
    use crate::repositories::SqlxTeamRepository;
    use domain::models::{Conference, Division, PlayoffResult, Team};
    use domain::repositories::TeamRepository;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        create_pool(&database_url)
            .await
            .expect("Failed to create pool")
    }

    async fn cleanup(pool: &PgPool) {
        sqlx::query!("DELETE FROM team_seasons")
            .execute(pool)
            .await
            .expect("Failed to cleanup team_seasons");
        sqlx::query!("DELETE FROM teams")
            .execute(pool)
            .await
            .expect("Failed to cleanup teams");
    }

    async fn create_test_team(pool: &PgPool) -> Team {
        let team_repo = SqlxTeamRepository::new(pool.clone());
        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();
        team_repo.create(&team).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_team_season() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let team = create_test_team(&pool).await;
        let repo = SqlxTeamSeasonRepository::new(pool.clone());

        let season = TeamSeason::new(
            team.id,
            2025,
            10,
            7,
            0,
            Some(PlayoffResult::WildCard),
            Some(15),
        )
        .unwrap();

        let result = repo.create(&season).await;
        assert!(result.is_ok());

        let created = result.unwrap();
        assert_eq!(created.team_id, team.id);
        assert_eq!(created.season_year, 2025);
        assert_eq!(created.wins, 10);
        assert_eq!(created.losses, 7);
        assert_eq!(created.ties, 0);
        assert_eq!(created.playoff_result, Some(PlayoffResult::WildCard));
        assert_eq!(created.draft_position, Some(15));

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_create_duplicate_fails() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let team = create_test_team(&pool).await;
        let repo = SqlxTeamSeasonRepository::new(pool.clone());

        let season1 = TeamSeason::new(team.id, 2025, 10, 7, 0, None, None).unwrap();
        repo.create(&season1)
            .await
            .expect("First create should succeed");

        let season2 = TeamSeason::new(team.id, 2025, 8, 9, 0, None, None).unwrap();
        let result = repo.create(&season2).await;
        assert!(result.is_err());

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_upsert() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let team = create_test_team(&pool).await;
        let repo = SqlxTeamSeasonRepository::new(pool.clone());

        let season1 = TeamSeason::new(team.id, 2025, 10, 7, 0, None, Some(15)).unwrap();
        let created = repo.upsert(&season1).await.unwrap();
        assert_eq!(created.wins, 10);

        // Upsert with updated data
        let season2 = TeamSeason::new(
            team.id,
            2025,
            11,
            6,
            0,
            Some(PlayoffResult::Divisional),
            Some(12),
        )
        .unwrap();
        let updated = repo.upsert(&season2).await.unwrap();
        assert_eq!(updated.wins, 11);
        assert_eq!(updated.losses, 6);
        assert_eq!(updated.playoff_result, Some(PlayoffResult::Divisional));
        assert_eq!(updated.draft_position, Some(12));

        // Should still be only one record
        let all = repo.find_by_year(2025).await.unwrap();
        assert_eq!(all.len(), 1);

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let team = create_test_team(&pool).await;
        let repo = SqlxTeamSeasonRepository::new(pool.clone());

        let season = TeamSeason::new(team.id, 2025, 10, 7, 0, None, None).unwrap();
        let created = repo.create(&season).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().wins, 10);

        let not_found = repo.find_by_id(Uuid::new_v4()).await.unwrap();
        assert!(not_found.is_none());

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_team_and_year() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let team = create_test_team(&pool).await;
        let repo = SqlxTeamSeasonRepository::new(pool.clone());

        let season = TeamSeason::new(team.id, 2025, 10, 7, 0, None, None).unwrap();
        repo.create(&season).await.unwrap();

        let found = repo.find_by_team_and_year(team.id, 2025).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().wins, 10);

        let not_found = repo.find_by_team_and_year(team.id, 2024).await.unwrap();
        assert!(not_found.is_none());

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_year_ordered_by_draft_position() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let team_repo = SqlxTeamRepository::new(pool.clone());
        let season_repo = SqlxTeamSeasonRepository::new(pool.clone());

        // Create multiple teams with different draft positions
        let team1 = Team::new(
            "Team A".to_string(),
            "A".to_string(),
            "City A".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();
        let team1 = team_repo.create(&team1).await.unwrap();

        let team2 = Team::new(
            "Team B".to_string(),
            "B".to_string(),
            "City B".to_string(),
            Conference::NFC,
            Division::NFCWest,
        )
        .unwrap();
        let team2 = team_repo.create(&team2).await.unwrap();

        let team3 = Team::new(
            "Team C".to_string(),
            "C".to_string(),
            "City C".to_string(),
            Conference::AFC,
            Division::AFCEast,
        )
        .unwrap();
        let team3 = team_repo.create(&team3).await.unwrap();

        // Create seasons with draft positions out of order
        let season1 = TeamSeason::new(team1.id, 2025, 3, 14, 0, None, Some(3)).unwrap();
        let season2 = TeamSeason::new(team2.id, 2025, 4, 13, 0, None, Some(1)).unwrap();
        let season3 = TeamSeason::new(team3.id, 2025, 5, 12, 0, None, Some(2)).unwrap();

        season_repo.create(&season1).await.unwrap();
        season_repo.create(&season2).await.unwrap();
        season_repo.create(&season3).await.unwrap();

        let ordered = season_repo
            .find_by_year_ordered_by_draft_position(2025)
            .await
            .unwrap();

        assert_eq!(ordered.len(), 3);
        assert_eq!(ordered[0].draft_position, Some(1));
        assert_eq!(ordered[1].draft_position, Some(2));
        assert_eq!(ordered[2].draft_position, Some(3));

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_delete_by_year() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let team = create_test_team(&pool).await;
        let repo = SqlxTeamSeasonRepository::new(pool.clone());

        let season1 = TeamSeason::new(team.id, 2024, 10, 7, 0, None, None).unwrap();
        let season2 = TeamSeason::new(team.id, 2025, 8, 9, 0, None, None).unwrap();

        repo.create(&season1).await.unwrap();
        repo.create(&season2).await.unwrap();

        repo.delete_by_year(2024).await.unwrap();

        let remaining_2024 = repo.find_by_year(2024).await.unwrap();
        let remaining_2025 = repo.find_by_year(2025).await.unwrap();

        assert_eq!(remaining_2024.len(), 0);
        assert_eq!(remaining_2025.len(), 1);

        cleanup(&pool).await;
    }

    #[tokio::test]
    async fn test_delete() {
        let pool = setup_test_pool().await;
        cleanup(&pool).await;

        let team = create_test_team(&pool).await;
        let repo = SqlxTeamSeasonRepository::new(pool.clone());

        let season = TeamSeason::new(team.id, 2025, 10, 7, 0, None, None).unwrap();
        let created = repo.create(&season).await.unwrap();

        let result = repo.delete(created.id).await;
        assert!(result.is_ok());

        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_none());

        let result = repo.delete(Uuid::new_v4()).await;
        assert!(result.is_err());

        cleanup(&pool).await;
    }
}
