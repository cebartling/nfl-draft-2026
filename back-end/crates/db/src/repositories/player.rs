use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::{Player, Position};
use domain::repositories::PlayerRepository;

use crate::errors::DbError;
use crate::models::PlayerDb;

/// SQLx implementation of PlayerRepository
pub struct SqlxPlayerRepository {
    pool: PgPool,
}

impl SqlxPlayerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PlayerRepository for SqlxPlayerRepository {
    async fn create(&self, player: &Player) -> DomainResult<Player> {
        let player_db = PlayerDb::from_domain(player);

        let result = sqlx::query_as!(
            PlayerDb,
            r#"
            INSERT INTO players (id, first_name, last_name, position, college, height_inches, weight_pounds, draft_year, draft_eligible, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, first_name, last_name, position, college, height_inches, weight_pounds, draft_year, draft_eligible, created_at, updated_at
            "#,
            player_db.id,
            player_db.first_name,
            player_db.last_name,
            player_db.position,
            player_db.college,
            player_db.height_inches,
            player_db.weight_pounds,
            player_db.draft_year,
            player_db.draft_eligible,
            player_db.created_at,
            player_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Player>> {
        let result = sqlx::query_as!(
            PlayerDb,
            r#"
            SELECT id, first_name, last_name, position, college, height_inches, weight_pounds, draft_year, draft_eligible, created_at, updated_at
            FROM players
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(player_db) => Ok(Some(player_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> DomainResult<Vec<Player>> {
        let results = sqlx::query_as!(
            PlayerDb,
            r#"
            SELECT id, first_name, last_name, position, college, height_inches, weight_pounds, draft_year, draft_eligible, created_at, updated_at
            FROM players
            ORDER BY last_name, first_name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|player_db| player_db.to_domain().map_err(Into::into))
            .collect()
    }

    async fn find_by_position(&self, position: Position) -> DomainResult<Vec<Player>> {
        let position_str = crate::models::player::position_to_string(&position);

        let results = sqlx::query_as!(
            PlayerDb,
            r#"
            SELECT id, first_name, last_name, position, college, height_inches, weight_pounds, draft_year, draft_eligible, created_at, updated_at
            FROM players
            WHERE position = $1
            ORDER BY last_name, first_name
            "#,
            position_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|player_db| player_db.to_domain().map_err(Into::into))
            .collect()
    }

    async fn find_by_draft_year(&self, year: i32) -> DomainResult<Vec<Player>> {
        let results = sqlx::query_as!(
            PlayerDb,
            r#"
            SELECT id, first_name, last_name, position, college, height_inches, weight_pounds, draft_year, draft_eligible, created_at, updated_at
            FROM players
            WHERE draft_year = $1
            ORDER BY last_name, first_name
            "#,
            year
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|player_db| player_db.to_domain().map_err(Into::into))
            .collect()
    }

    async fn find_draft_eligible(&self, year: i32) -> DomainResult<Vec<Player>> {
        let results = sqlx::query_as!(
            PlayerDb,
            r#"
            SELECT id, first_name, last_name, position, college, height_inches, weight_pounds, draft_year, draft_eligible, created_at, updated_at
            FROM players
            WHERE draft_eligible = true AND draft_year = $1
            ORDER BY last_name, first_name
            "#,
            year
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|player_db| player_db.to_domain().map_err(Into::into))
            .collect()
    }

    async fn update(&self, player: &Player) -> DomainResult<Player> {
        let player_db = PlayerDb::from_domain(player);

        let result = sqlx::query_as!(
            PlayerDb,
            r#"
            UPDATE players
            SET first_name = $2, last_name = $3, position = $4, college = $5,
                height_inches = $6, weight_pounds = $7, draft_year = $8,
                draft_eligible = $9, updated_at = NOW()
            WHERE id = $1
            RETURNING id, first_name, last_name, position, college, height_inches, weight_pounds, draft_year, draft_eligible, created_at, updated_at
            "#,
            player_db.id,
            player_db.first_name,
            player_db.last_name,
            player_db.position,
            player_db.college,
            player_db.height_inches,
            player_db.weight_pounds,
            player_db.draft_year,
            player_db.draft_eligible
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                return DbError::NotFound(format!("Player with id {} not found", player_db.id));
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM players
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("Player with id {} not found", id)).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_pool;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
        });

        create_pool(&database_url)
            .await
            .expect("Failed to create pool")
    }

    async fn cleanup_players(pool: &PgPool) {
        sqlx::query!("DELETE FROM players")
            .execute(pool)
            .await
            .expect("Failed to cleanup players");
    }

    #[tokio::test]
    async fn test_create_player() {
        let pool = setup_test_pool().await;
        cleanup_players(&pool).await;

        let repo = SqlxPlayerRepository::new(pool.clone());

        let player =
            Player::new("John".to_string(), "Doe".to_string(), Position::QB, 2026).unwrap();

        let result = repo.create(&player).await;
        assert!(result.is_ok());

        let created = result.unwrap();
        assert_eq!(created.first_name, "John");
        assert_eq!(created.last_name, "Doe");
        assert_eq!(created.position, Position::QB);

        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = setup_test_pool().await;
        cleanup_players(&pool).await;

        let repo = SqlxPlayerRepository::new(pool.clone());

        let player =
            Player::new("John".to_string(), "Doe".to_string(), Position::QB, 2026).unwrap();

        let created = repo.create(&player).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().full_name(), "John Doe");

        // Test not found
        let not_found = repo.find_by_id(Uuid::new_v4()).await.unwrap();
        assert!(not_found.is_none());

        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_find_all() {
        let pool = setup_test_pool().await;
        cleanup_players(&pool).await;

        let repo = SqlxPlayerRepository::new(pool.clone());

        let player1 =
            Player::new("John".to_string(), "Doe".to_string(), Position::QB, 2026).unwrap();
        let player2 =
            Player::new("Jane".to_string(), "Smith".to_string(), Position::WR, 2026).unwrap();

        repo.create(&player1).await.unwrap();
        repo.create(&player2).await.unwrap();

        let players = repo.find_all().await.unwrap();
        assert_eq!(players.len(), 2);

        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_position() {
        let pool = setup_test_pool().await;
        cleanup_players(&pool).await;

        let repo = SqlxPlayerRepository::new(pool.clone());

        let qb = Player::new("John".to_string(), "Doe".to_string(), Position::QB, 2026).unwrap();
        let wr = Player::new("Jane".to_string(), "Smith".to_string(), Position::WR, 2026).unwrap();

        repo.create(&qb).await.unwrap();
        repo.create(&wr).await.unwrap();

        let qbs = repo.find_by_position(Position::QB).await.unwrap();
        assert_eq!(qbs.len(), 1);
        assert_eq!(qbs[0].position, Position::QB);

        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_draft_year() {
        let pool = setup_test_pool().await;
        cleanup_players(&pool).await;

        let repo = SqlxPlayerRepository::new(pool.clone());

        let player2026 =
            Player::new("John".to_string(), "Doe".to_string(), Position::QB, 2026).unwrap();
        let player2027 =
            Player::new("Jane".to_string(), "Smith".to_string(), Position::WR, 2027).unwrap();

        repo.create(&player2026).await.unwrap();
        repo.create(&player2027).await.unwrap();

        let players_2026 = repo.find_by_draft_year(2026).await.unwrap();
        assert_eq!(players_2026.len(), 1);
        assert_eq!(players_2026[0].draft_year, 2026);

        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_find_draft_eligible() {
        let pool = setup_test_pool().await;
        cleanup_players(&pool).await;

        let repo = SqlxPlayerRepository::new(pool.clone());

        let eligible =
            Player::new("John".to_string(), "Doe".to_string(), Position::QB, 2026).unwrap();
        let mut not_eligible =
            Player::new("Jane".to_string(), "Smith".to_string(), Position::WR, 2026).unwrap();
        not_eligible.draft_eligible = false;

        repo.create(&eligible).await.unwrap();
        repo.create(&not_eligible).await.unwrap();

        let draft_eligible = repo.find_draft_eligible(2026).await.unwrap();
        assert_eq!(draft_eligible.len(), 1);
        assert_eq!(draft_eligible[0].first_name, "John");

        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_update_player() {
        let pool = setup_test_pool().await;
        cleanup_players(&pool).await;

        let repo = SqlxPlayerRepository::new(pool.clone());

        let player =
            Player::new("John".to_string(), "Doe".to_string(), Position::QB, 2026).unwrap();
        let mut created = repo.create(&player).await.unwrap();

        // Update the player
        created.first_name = "Johnny".to_string();
        let updated = repo.update(&created).await.unwrap();

        assert_eq!(updated.first_name, "Johnny");
        assert_ne!(updated.updated_at, updated.created_at);

        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_delete_player() {
        let pool = setup_test_pool().await;
        cleanup_players(&pool).await;

        let repo = SqlxPlayerRepository::new(pool.clone());

        let player =
            Player::new("John".to_string(), "Doe".to_string(), Position::QB, 2026).unwrap();
        let created = repo.create(&player).await.unwrap();

        // Delete the player
        let result = repo.delete(created.id).await;
        assert!(result.is_ok());

        // Verify it's gone
        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_none());

        // Delete non-existent player should fail
        let result = repo.delete(Uuid::new_v4()).await;
        assert!(result.is_err());

        cleanup_players(&pool).await;
    }
}
