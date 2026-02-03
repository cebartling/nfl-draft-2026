use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::CombineResults;
use domain::repositories::CombineResultsRepository;

use crate::errors::DbError;
use crate::models::CombineResultsDb;

/// SQLx implementation of CombineResultsRepository
pub struct SqlxCombineResultsRepository {
    pool: PgPool,
}

impl SqlxCombineResultsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CombineResultsRepository for SqlxCombineResultsRepository {
    async fn create(&self, results: &CombineResults) -> DomainResult<CombineResults> {
        let results_db = CombineResultsDb::from_domain(results);

        let result = sqlx::query_as!(
            CombineResultsDb,
            r#"
            INSERT INTO combine_results
            (id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
             broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
                      broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at
            "#,
            results_db.id,
            results_db.player_id,
            results_db.year,
            results_db.forty_yard_dash,
            results_db.bench_press,
            results_db.vertical_jump,
            results_db.broad_jump,
            results_db.three_cone_drill,
            results_db.twenty_yard_shuttle,
            results_db.created_at,
            results_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(format!(
                        "Combine results for player {} in year {} already exist",
                        results.player_id, results.year
                    ));
                }
                if db_err.is_foreign_key_violation() {
                    return DbError::NotFound(format!("Player with id {} not found", results.player_id));
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<CombineResults>> {
        let result = sqlx::query_as!(
            CombineResultsDb,
            r#"
            SELECT id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
                   broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at
            FROM combine_results
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(results_db) => Ok(Some(results_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_player_id(&self, player_id: Uuid) -> DomainResult<Vec<CombineResults>> {
        let results = sqlx::query_as!(
            CombineResultsDb,
            r#"
            SELECT id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
                   broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at
            FROM combine_results
            WHERE player_id = $1
            ORDER BY year DESC
            "#,
            player_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|r| r.to_domain().map_err(Into::into))
            .collect()
    }

    async fn find_by_player_and_year(
        &self,
        player_id: Uuid,
        year: i32,
    ) -> DomainResult<Option<CombineResults>> {
        let result = sqlx::query_as!(
            CombineResultsDb,
            r#"
            SELECT id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
                   broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at
            FROM combine_results
            WHERE player_id = $1 AND year = $2
            "#,
            player_id,
            year
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(results_db) => Ok(Some(results_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn update(&self, results: &CombineResults) -> DomainResult<CombineResults> {
        let results_db = CombineResultsDb::from_domain(results);

        let result = sqlx::query_as!(
            CombineResultsDb,
            r#"
            UPDATE combine_results
            SET forty_yard_dash = $2,
                bench_press = $3,
                vertical_jump = $4,
                broad_jump = $5,
                three_cone_drill = $6,
                twenty_yard_shuttle = $7,
                updated_at = $8
            WHERE id = $1
            RETURNING id, player_id, year, forty_yard_dash, bench_press, vertical_jump,
                   broad_jump, three_cone_drill, twenty_yard_shuttle, created_at, updated_at
            "#,
            results_db.id,
            results_db.forty_yard_dash,
            results_db.bench_press,
            results_db.vertical_jump,
            results_db.broad_jump,
            results_db.three_cone_drill,
            results_db.twenty_yard_shuttle,
            results_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        result.to_domain().map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM combine_results WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_pool;
    use domain::models::Player;
    use domain::repositories::PlayerRepository;
    use crate::repositories::SqlxPlayerRepository;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
            });

        create_pool(&database_url).await.expect("Failed to create pool")
    }

    async fn cleanup_combine_results(pool: &PgPool) {
        sqlx::query!("DELETE FROM combine_results")
            .execute(pool)
            .await
            .expect("Failed to cleanup combine_results");
    }

    async fn cleanup_players(pool: &PgPool) {
        sqlx::query!("DELETE FROM players")
            .execute(pool)
            .await
            .expect("Failed to cleanup players");
    }

    async fn create_test_player(pool: &PgPool) -> Player {
        let player_repo = SqlxPlayerRepository::new(pool.clone());
        let player = Player::new(
            "Test".to_string(),
            "Player".to_string(),
            domain::models::Position::QB,
            2026,
        )
        .unwrap();
        player_repo.create(&player).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_combine_results() {
        let pool = setup_test_pool().await;
        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;

        let player = create_test_player(&pool).await;
        let repo = SqlxCombineResultsRepository::new(pool.clone());

        let results = CombineResults::new(player.id, 2026)
            .unwrap()
            .with_forty_yard_dash(4.52)
            .unwrap()
            .with_bench_press(20)
            .unwrap();

        let created = repo.create(&results).await.unwrap();

        assert_eq!(created.player_id, player.id);
        assert_eq!(created.year, 2026);
        assert_eq!(created.forty_yard_dash, Some(4.52));
        assert_eq!(created.bench_press, Some(20));

        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = setup_test_pool().await;
        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;

        let player = create_test_player(&pool).await;
        let repo = SqlxCombineResultsRepository::new(pool.clone());

        let results = CombineResults::new(player.id, 2026)
            .unwrap()
            .with_forty_yard_dash(4.52)
            .unwrap();

        let created = repo.create(&results).await.unwrap();
        let found = repo.find_by_id(created.id).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.forty_yard_dash, Some(4.52));

        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_player_id() {
        let pool = setup_test_pool().await;
        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;

        let player = create_test_player(&pool).await;
        let repo = SqlxCombineResultsRepository::new(pool.clone());

        let results1 = CombineResults::new(player.id, 2025).unwrap();
        let results2 = CombineResults::new(player.id, 2026).unwrap();

        repo.create(&results1).await.unwrap();
        repo.create(&results2).await.unwrap();

        let found = repo.find_by_player_id(player.id).await.unwrap();

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].year, 2026); // Most recent first
        assert_eq!(found[1].year, 2025);

        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_player_and_year() {
        let pool = setup_test_pool().await;
        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;

        let player = create_test_player(&pool).await;
        let repo = SqlxCombineResultsRepository::new(pool.clone());

        let results = CombineResults::new(player.id, 2026).unwrap();
        repo.create(&results).await.unwrap();

        let found = repo.find_by_player_and_year(player.id, 2026).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.player_id, player.id);
        assert_eq!(found.year, 2026);

        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_update_combine_results() {
        let pool = setup_test_pool().await;
        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;

        let player = create_test_player(&pool).await;
        let repo = SqlxCombineResultsRepository::new(pool.clone());

        let results = CombineResults::new(player.id, 2026).unwrap();
        let created = repo.create(&results).await.unwrap();

        let updated = CombineResults {
            forty_yard_dash: Some(4.52),
            bench_press: Some(20),
            ..created
        };

        let result = repo.update(&updated).await.unwrap();

        assert_eq!(result.forty_yard_dash, Some(4.52));
        assert_eq!(result.bench_press, Some(20));

        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_delete_combine_results() {
        let pool = setup_test_pool().await;
        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;

        let player = create_test_player(&pool).await;
        let repo = SqlxCombineResultsRepository::new(pool.clone());

        let results = CombineResults::new(player.id, 2026).unwrap();
        let created = repo.create(&results).await.unwrap();

        repo.delete(created.id).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_none());

        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;
    }

    #[tokio::test]
    async fn test_duplicate_player_year() {
        let pool = setup_test_pool().await;
        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;

        let player = create_test_player(&pool).await;
        let repo = SqlxCombineResultsRepository::new(pool.clone());

        let results = CombineResults::new(player.id, 2026).unwrap();
        repo.create(&results).await.unwrap();

        let duplicate = CombineResults::new(player.id, 2026).unwrap();
        let result = repo.create(&duplicate).await;

        assert!(result.is_err());

        cleanup_combine_results(&pool).await;
        cleanup_players(&pool).await;
    }
}
