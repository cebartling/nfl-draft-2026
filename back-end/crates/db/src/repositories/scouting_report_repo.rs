use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use domain::errors::DomainResult;
use domain::models::ScoutingReport;
use domain::repositories::ScoutingReportRepository;

use crate::errors::DbError;
use crate::models::ScoutingReportDb;

/// SQLx implementation of ScoutingReportRepository
pub struct SqlxScoutingReportRepository {
    pool: PgPool,
}

impl SqlxScoutingReportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ScoutingReportRepository for SqlxScoutingReportRepository {
    async fn create(&self, report: &ScoutingReport) -> DomainResult<ScoutingReport> {
        let report_db = ScoutingReportDb::from_domain(report);

        let result = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            INSERT INTO scouting_reports
            (id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            "#,
            report_db.id,
            report_db.player_id,
            report_db.team_id,
            report_db.grade,
            report_db.notes,
            report_db.fit_grade,
            report_db.injury_concern,
            report_db.character_concern,
            report_db.created_at,
            report_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return DbError::DuplicateEntry(format!(
                        "Scouting report for team {} and player {} already exists",
                        report.team_id, report.player_id
                    ));
                }
                if db_err.is_foreign_key_violation() {
                    return DbError::NotFound("Team or player not found".to_string());
                }
            }
            DbError::DatabaseError(e)
        })?;

        result.to_domain().map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<ScoutingReport>> {
        let result = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            SELECT id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            FROM scouting_reports
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(report_db) => Ok(Some(report_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn find_by_team_id(&self, team_id: Uuid) -> DomainResult<Vec<ScoutingReport>> {
        let results = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            SELECT id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            FROM scouting_reports
            WHERE team_id = $1
            ORDER BY grade DESC
            "#,
            team_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        results
            .into_iter()
            .map(|r| r.to_domain().map_err(Into::into))
            .collect()
    }

    async fn find_by_player_id(&self, player_id: Uuid) -> DomainResult<Vec<ScoutingReport>> {
        let results = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            SELECT id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            FROM scouting_reports
            WHERE player_id = $1
            ORDER BY grade DESC
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

    async fn find_by_team_and_player(
        &self,
        team_id: Uuid,
        player_id: Uuid,
    ) -> DomainResult<Option<ScoutingReport>> {
        let result = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            SELECT id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            FROM scouting_reports
            WHERE team_id = $1 AND player_id = $2
            "#,
            team_id,
            player_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        match result {
            Some(report_db) => Ok(Some(report_db.to_domain()?)),
            None => Ok(None),
        }
    }

    async fn update(&self, report: &ScoutingReport) -> DomainResult<ScoutingReport> {
        let report_db = ScoutingReportDb::from_domain(report);

        let result = sqlx::query_as!(
            ScoutingReportDb,
            r#"
            UPDATE scouting_reports
            SET grade = $2,
                notes = $3,
                fit_grade = $4,
                injury_concern = $5,
                character_concern = $6,
                updated_at = $7
            WHERE id = $1
            RETURNING id, player_id, team_id, grade, notes, fit_grade, injury_concern, character_concern, created_at, updated_at
            "#,
            report_db.id,
            report_db.grade,
            report_db.notes,
            report_db.fit_grade,
            report_db.injury_concern,
            report_db.character_concern,
            report_db.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::DatabaseError)?;

        result.to_domain().map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM scouting_reports WHERE id = $1
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
    use domain::models::{Player, Team, Conference, Division, FitGrade};
    use domain::repositories::{PlayerRepository, TeamRepository};
    use crate::repositories::{SqlxPlayerRepository, SqlxTeamRepository};

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgresql://nfl_draft_user:nfl_draft_pass@localhost:5432/nfl_draft_test".to_string()
            });

        create_pool(&database_url).await.expect("Failed to create pool")
    }

    async fn cleanup_scouting_reports(pool: &PgPool) {
        sqlx::query!("DELETE FROM scouting_reports")
            .execute(pool)
            .await
            .expect("Failed to cleanup scouting_reports");
    }

    async fn cleanup_players(pool: &PgPool) {
        sqlx::query!("DELETE FROM players")
            .execute(pool)
            .await
            .expect("Failed to cleanup players");
    }

    async fn cleanup_teams(pool: &PgPool) {
        sqlx::query!("DELETE FROM teams")
            .execute(pool)
            .await
            .expect("Failed to cleanup teams");
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

    async fn create_test_team(pool: &PgPool, abbr: &str) -> Team {
        let team_repo = SqlxTeamRepository::new(pool.clone());
        let team = Team::new(
            format!("Test Team {}", abbr),
            abbr.to_string(),
            "Test City".to_string(),
            Conference::AFC,
            Division::AFCEast,
        )
        .unwrap();
        team_repo.create(&team).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_scouting_report() {
        let pool = setup_test_pool().await;
        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;

        let player = create_test_player(&pool).await;
        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxScoutingReportRepository::new(pool.clone());

        let report = ScoutingReport::new(player.id, team.id, 8.5)
            .unwrap()
            .with_fit_grade(FitGrade::A)
            .with_notes("Excellent prospect".to_string())
            .unwrap();

        let created = repo.create(&report).await.unwrap();

        assert_eq!(created.player_id, player.id);
        assert_eq!(created.team_id, team.id);
        assert_eq!(created.grade, 8.5);
        assert_eq!(created.fit_grade, Some(FitGrade::A));

        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let pool = setup_test_pool().await;
        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;

        let player = create_test_player(&pool).await;
        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxScoutingReportRepository::new(pool.clone());

        let report = ScoutingReport::new(player.id, team.id, 8.5).unwrap();
        let created = repo.create(&report).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.grade, 8.5);

        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_team_id() {
        let pool = setup_test_pool().await;
        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;

        let player1 = create_test_player(&pool).await;
        let player_repo = SqlxPlayerRepository::new(pool.clone());
        let player2 = player_repo.create(&Player::new(
            "Second".to_string(),
            "Player".to_string(),
            domain::models::Position::RB,
            2026,
        ).unwrap()).await.unwrap();

        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxScoutingReportRepository::new(pool.clone());

        let report1 = ScoutingReport::new(player1.id, team.id, 9.0).unwrap();
        let report2 = ScoutingReport::new(player2.id, team.id, 7.5).unwrap();

        repo.create(&report1).await.unwrap();
        repo.create(&report2).await.unwrap();

        let found = repo.find_by_team_id(team.id).await.unwrap();

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].grade, 9.0); // Highest grade first
        assert_eq!(found[1].grade, 7.5);

        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_player_id() {
        let pool = setup_test_pool().await;
        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;

        let player = create_test_player(&pool).await;
        let team1 = create_test_team(&pool, "TS1").await;
        let team2 = create_test_team(&pool, "TS2").await;
        let repo = SqlxScoutingReportRepository::new(pool.clone());

        let report1 = ScoutingReport::new(player.id, team1.id, 9.0).unwrap();
        let report2 = ScoutingReport::new(player.id, team2.id, 7.5).unwrap();

        repo.create(&report1).await.unwrap();
        repo.create(&report2).await.unwrap();

        let found = repo.find_by_player_id(player.id).await.unwrap();

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].grade, 9.0);
        assert_eq!(found[1].grade, 7.5);

        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_find_by_team_and_player() {
        let pool = setup_test_pool().await;
        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;

        let player = create_test_player(&pool).await;
        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxScoutingReportRepository::new(pool.clone());

        let report = ScoutingReport::new(player.id, team.id, 8.5).unwrap();
        repo.create(&report).await.unwrap();

        let found = repo.find_by_team_and_player(team.id, player.id).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.player_id, player.id);
        assert_eq!(found.team_id, team.id);
        assert_eq!(found.grade, 8.5);

        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_update_scouting_report() {
        let pool = setup_test_pool().await;
        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;

        let player = create_test_player(&pool).await;
        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxScoutingReportRepository::new(pool.clone());

        let report = ScoutingReport::new(player.id, team.id, 8.5).unwrap();
        let created = repo.create(&report).await.unwrap();

        let updated = ScoutingReport {
            grade: 9.0,
            fit_grade: Some(FitGrade::A),
            notes: Some("Updated notes".to_string()),
            ..created
        };

        let result = repo.update(&updated).await.unwrap();

        assert_eq!(result.grade, 9.0);
        assert_eq!(result.fit_grade, Some(FitGrade::A));
        assert_eq!(result.notes, Some("Updated notes".to_string()));

        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_delete_scouting_report() {
        let pool = setup_test_pool().await;
        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;

        let player = create_test_player(&pool).await;
        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxScoutingReportRepository::new(pool.clone());

        let report = ScoutingReport::new(player.id, team.id, 8.5).unwrap();
        let created = repo.create(&report).await.unwrap();

        repo.delete(created.id).await.unwrap();

        let found = repo.find_by_id(created.id).await.unwrap();
        assert!(found.is_none());

        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;
    }

    #[tokio::test]
    async fn test_duplicate_team_player() {
        let pool = setup_test_pool().await;
        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;

        let player = create_test_player(&pool).await;
        let team = create_test_team(&pool, "TST").await;
        let repo = SqlxScoutingReportRepository::new(pool.clone());

        let report = ScoutingReport::new(player.id, team.id, 8.5).unwrap();
        repo.create(&report).await.unwrap();

        let duplicate = ScoutingReport::new(player.id, team.id, 9.0).unwrap();
        let result = repo.create(&duplicate).await;

        assert!(result.is_err());

        cleanup_scouting_reports(&pool).await;
        cleanup_players(&pool).await;
        cleanup_teams(&pool).await;
    }
}
