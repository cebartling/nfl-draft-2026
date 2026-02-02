use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::Team;
use domain::repositories::TeamRepository;

use crate::errors::DbError;
use crate::models::TeamDb;

/// SQLx implementation of TeamRepository
pub struct SqlxTeamRepository {
    pool: PgPool,
}

impl SqlxTeamRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TeamRepository for SqlxTeamRepository {
    async fn create(&self, team: &Team) -> DomainResult<Team> {
        let team_db = TeamDb::from_domain(team);

        let result = sqlx::query_as!(
            TeamDb,
            r#"
            INSERT INTO teams (id, name, abbreviation, city, conference, division, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, abbreviation, city, conference, division, created_at, updated_at
            "#,
            team_db.id,
            team_db.name,
            team_db.abbreviation,
            team_db.city,
            team_db.conference,
            team_db.division,
            team_db.created_at,
            team_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(format!(
                        "Team with abbreviation '{}' already exists",
                        team_db.abbreviation
                    ));
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Team>> {
        let result = sqlx::query_as!(
            TeamDb,
            r#"
            SELECT id, name, abbreviation, city, conference, division, created_at, updated_at
            FROM teams
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(team_db) => Ok(Some(team_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_abbreviation(&self, abbreviation: &str) -> DomainResult<Option<Team>> {
        let result = sqlx::query_as!(
            TeamDb,
            r#"
            SELECT id, name, abbreviation, city, conference, division, created_at, updated_at
            FROM teams
            WHERE abbreviation = $1
            "#,
            abbreviation
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(team_db) => Ok(Some(team_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> DomainResult<Vec<Team>> {
        let results = sqlx::query_as!(
            TeamDb,
            r#"
            SELECT id, name, abbreviation, city, conference, division, created_at, updated_at
            FROM teams
            ORDER BY conference, division, name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|team_db| team_db.to_domain().map_err(Into::into))
            .collect()
    }

    async fn update(&self, team: &Team) -> DomainResult<Team> {
        let team_db = TeamDb::from_domain(team);

        let result = sqlx::query_as!(
            TeamDb,
            r#"
            UPDATE teams
            SET name = $2, abbreviation = $3, city = $4, conference = $5, division = $6, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, abbreviation, city, conference, division, created_at, updated_at
            "#,
            team_db.id,
            team_db.name,
            team_db.abbreviation,
            team_db.city,
            team_db.conference,
            team_db.division
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                return DbError::NotFound(format!("Team with id {} not found", team_db.id));
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM teams
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("Team with id {} not found", id)).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::{Conference, Division};
    use crate::create_pool;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft".to_string()
            });

        create_pool(&database_url).await.expect("Failed to create pool")
    }

    async fn cleanup_teams(pool: &PgPool) {
        sqlx::query!("DELETE FROM teams")
            .execute(pool)
            .await
            .expect("Failed to cleanup teams");
    }

    #[tokio::test]
    async fn test_create_team() {
        let pool = setup_test_pool().await;
        cleanup_teams(&pool).await;

        let repo = SqlxTeamRepository::new(pool.clone());

        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();

        let result = repo.create(&team).await;
        assert!(result.is_ok());

        let created = result.unwrap();
        assert_eq!(created.name, "Dallas Cowboys");
        assert_eq!(created.abbreviation, "DAL");

        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_create_duplicate_abbreviation_fails() {
        let pool = setup_test_pool().await;
        cleanup_teams(&pool).await;

        let repo = SqlxTeamRepository::new(pool.clone());

        let team1 = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();

        repo.create(&team1).await.expect("First create should succeed");

        let team2 = Team::new(
            "Different Team".to_string(),
            "DAL".to_string(), // Same abbreviation
            "Different City".to_string(),
            Conference::NFC,
            Division::NFCWest,
        )
        .unwrap();

        let result = repo.create(&team2).await;
        assert!(result.is_err());

        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = setup_test_pool().await;
        cleanup_teams(&pool).await;

        let repo = SqlxTeamRepository::new(pool.clone());

        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();

        let created = repo.create(&team).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Dallas Cowboys");

        // Test not found
        let not_found = repo.find_by_id(Uuid::new_v4()).await.unwrap();
        assert!(not_found.is_none());

        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_abbreviation() {
        let pool = setup_test_pool().await;
        cleanup_teams(&pool).await;

        let repo = SqlxTeamRepository::new(pool.clone());

        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();

        repo.create(&team).await.unwrap();

        let found = repo.find_by_abbreviation("DAL").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Dallas Cowboys");

        // Test not found
        let not_found = repo.find_by_abbreviation("XXX").await.unwrap();
        assert!(not_found.is_none());

        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_all() {
        let pool = setup_test_pool().await;
        cleanup_teams(&pool).await;

        let repo = SqlxTeamRepository::new(pool.clone());

        let team1 = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();

        let team2 = Team::new(
            "Kansas City Chiefs".to_string(),
            "KC".to_string(),
            "Kansas City".to_string(),
            Conference::AFC,
            Division::AFCWest,
        )
        .unwrap();

        repo.create(&team1).await.unwrap();
        repo.create(&team2).await.unwrap();

        let teams = repo.find_all().await.unwrap();
        assert_eq!(teams.len(), 2);

        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_update_team() {
        let pool = setup_test_pool().await;
        cleanup_teams(&pool).await;

        let repo = SqlxTeamRepository::new(pool.clone());

        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();

        let mut created = repo.create(&team).await.unwrap();

        // Update the team
        created.name = "Dallas Cowboys Updated".to_string();
        let updated = repo.update(&created).await.unwrap();

        assert_eq!(updated.name, "Dallas Cowboys Updated");
        assert_ne!(updated.updated_at, updated.created_at);

        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_delete_team() {
        let pool = setup_test_pool().await;
        cleanup_teams(&pool).await;

        let repo = SqlxTeamRepository::new(pool.clone());

        let team = Team::new(
            "Dallas Cowboys".to_string(),
            "DAL".to_string(),
            "Dallas".to_string(),
            Conference::NFC,
            Division::NFCEast,
        )
        .unwrap();

        let created = repo.create(&team).await.unwrap();

        // Delete the team
        let result = repo.delete(created.id).await;
        assert!(result.is_ok());

        // Verify it's gone
        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_none());

        // Delete non-existent team should fail
        let result = repo.delete(Uuid::new_v4()).await;
        assert!(result.is_err());

        cleanup_teams(&pool).await;
    }
}
