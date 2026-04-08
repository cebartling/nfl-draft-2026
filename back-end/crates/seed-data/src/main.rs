use seed_data::{
    combine_loader, draft_order_loader, draft_order_validator, feldman_freak_loader,
    feldman_freak_validator, loader, percentile_loader, rankings_loader, rankings_validator,
    scouting_report_loader, scouting_report_validator, team_loader, team_need_loader,
    team_need_validator, team_season_loader, team_season_validator, team_validator,
    the_beast_loader, validator,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use db::{
    create_pool,
    repositories::{
        SqlxCombinePercentileRepository, SqlxCombineResultsRepository, SqlxDraftPickRepository,
        SqlxDraftRepository, SqlxFeldmanFreakRepository, SqlxPlayerRepository,
        SqlxProspectProfileRepository, SqlxProspectRankingRepository, SqlxRankingSourceRepository,
        SqlxScoutingReportRepository, SqlxTeamNeedRepository, SqlxTeamRepository,
        SqlxTeamSeasonRepository,
    },
};
use domain::repositories::PlayerRepository;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "seed-data")]
#[command(about = "Seed NFL Draft data into the database")]
struct Cli {
    #[command(subcommand)]
    entity: EntityCommands,
}

#[derive(Subcommand)]
enum EntityCommands {
    /// Manage player data
    Players {
        #[command(subcommand)]
        action: PlayerActions,
    },

    /// Manage team data
    Teams {
        #[command(subcommand)]
        action: TeamActions,
    },

    /// Manage team positional draft needs
    Needs {
        #[command(subcommand)]
        action: NeedActions,
    },

    /// Manage team season records and draft positions
    Seasons {
        #[command(subcommand)]
        action: SeasonActions,
    },

    /// Manage draft order data
    DraftOrder {
        #[command(subcommand)]
        action: DraftOrderActions,
    },

    /// Manage scouting reports from prospect rankings
    Scouting {
        #[command(subcommand)]
        action: ScoutingActions,
    },

    /// Manage prospect rankings from external sources (big boards)
    Rankings {
        #[command(subcommand)]
        action: RankingsActions,
    },

    /// Manage Feldman Freaks list data
    Freaks {
        #[command(subcommand)]
        action: FreaksActions,
    },

    /// Manage combine results data
    Combine {
        #[command(subcommand)]
        action: CombineActions,
    },

    /// Manage combine percentile reference data
    Percentiles {
        #[command(subcommand)]
        action: PercentilesActions,
    },

    /// Manage Dane Brugler's The Beast 2026 prospect profiles
    TheBeast {
        #[command(subcommand)]
        action: TheBeastActions,
    },
}

#[derive(Subcommand)]
enum TheBeastActions {
    /// Load The Beast 2026 JSON file (output of the Bun scraper) into the database
    Load {
        /// Path to the JSON data file produced by `bun run scrape the-beast`
        #[arg(short, long, default_value = "data/the_beast_2026.json")]
        file: String,

        /// Simulate loading without writing to the database
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum PlayerActions {
    /// Load players from JSON file into the database
    Load {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/players_2026.json")]
        file: String,

        /// Simulate loading without writing to database
        #[arg(long)]
        dry_run: bool,
    },

    /// Clear all players for a given draft year
    Clear {
        /// The draft year to clear
        #[arg(short, long)]
        year: i32,
    },

    /// Validate JSON file without loading
    Validate {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/players_2026.json")]
        file: String,
    },
}

#[derive(Subcommand)]
enum TeamActions {
    /// Load teams from JSON file into the database
    Load {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/teams_nfl.json")]
        file: String,

        /// Simulate loading without writing to database
        #[arg(long)]
        dry_run: bool,
    },

    /// Clear all teams from the database
    Clear,

    /// Validate JSON file without loading
    Validate {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/teams_nfl.json")]
        file: String,
    },
}

#[derive(Subcommand)]
enum NeedActions {
    /// Load team needs from JSON file into the database
    Load {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/team_needs_2026.json")]
        file: String,

        /// Simulate loading without writing to database
        #[arg(long)]
        dry_run: bool,
    },

    /// Clear all team needs from the database
    Clear,

    /// Validate JSON file without loading
    Validate {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/team_needs_2026.json")]
        file: String,
    },
}

#[derive(Subcommand)]
enum DraftOrderActions {
    /// Load draft order from JSON file into the database
    Load {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/draft_order_2026.json")]
        file: String,

        /// Simulate loading without writing to database
        #[arg(long)]
        dry_run: bool,
    },

    /// Clear draft order for a given year
    Clear {
        /// The draft year to clear
        #[arg(short, long)]
        year: i32,
    },

    /// Validate JSON file without loading
    Validate {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/draft_order_2026.json")]
        file: String,
    },
}

#[derive(Subcommand)]
enum SeasonActions {
    /// Load team seasons from JSON file into the database
    Load {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/team_seasons_2025.json")]
        file: String,

        /// Simulate loading without writing to database
        #[arg(long)]
        dry_run: bool,
    },

    /// Clear all team seasons for a given year
    Clear {
        /// The season year to clear
        #[arg(short, long)]
        year: i32,
    },

    /// Validate JSON file without loading
    Validate {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/team_seasons_2025.json")]
        file: String,
    },
}

#[derive(Subcommand)]
enum ScoutingActions {
    /// Load scouting reports from prospect rankings JSON file
    Load {
        /// Path to the rankings JSON data file
        #[arg(short, long, default_value = "data/rankings/rankings.json")]
        file: String,

        /// Simulate loading without writing to database
        #[arg(long)]
        dry_run: bool,
    },

    /// Clear all scouting reports for a draft year
    Clear {
        /// The draft year to clear
        #[arg(short, long)]
        year: i32,
    },

    /// Validate rankings JSON file without loading
    Validate {
        /// Path to the rankings JSON data file
        #[arg(short, long, default_value = "data/rankings/rankings.json")]
        file: String,
    },
}

#[derive(Subcommand)]
enum FreaksActions {
    /// Load Feldman Freaks from JSON file into the database
    Load {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/feldman_freaks_2026.json")]
        file: String,

        /// Simulate loading without writing to database
        #[arg(long)]
        dry_run: bool,
    },

    /// Clear Feldman Freaks for a given year
    Clear {
        /// The year to clear
        #[arg(short, long)]
        year: i32,
    },

    /// Validate JSON file without loading
    Validate {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/feldman_freaks_2026.json")]
        file: String,
    },
}

#[derive(Subcommand)]
enum CombineActions {
    /// Load combine results from JSON file into the database
    Load {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/combine_2026.json")]
        file: String,

        /// Simulate loading without writing to database
        #[arg(long)]
        dry_run: bool,
    },

    /// Clear combine results for a given year
    Clear {
        /// The year to clear
        #[arg(short, long)]
        year: i32,
    },

    /// Validate JSON file without loading
    Validate {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/combine_2026.json")]
        file: String,
    },
}

#[derive(Subcommand)]
enum RankingsActions {
    /// Load prospect rankings from JSON file (auto-creates new players + scouting reports)
    Load {
        /// Path to the rankings JSON data file
        #[arg(short, long)]
        file: String,

        /// Simulate loading without writing to database
        #[arg(long)]
        dry_run: bool,
    },

    /// Clear rankings for a source
    Clear {
        /// The ranking source name (e.g., "Tankathon", "Walter Football")
        #[arg(short, long)]
        source: String,
    },

    /// Validate rankings JSON file without loading
    Validate {
        /// Path to the rankings JSON data file
        #[arg(short, long)]
        file: String,
    },
}

#[derive(Subcommand)]
enum PercentilesActions {
    /// Load combine percentiles from JSON file into the database
    Load {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/combine_percentiles.json")]
        file: String,
    },

    /// Validate JSON file without loading
    Validate {
        /// Path to the JSON data file
        #[arg(short, long, default_value = "data/combine_percentiles.json")]
        file: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.entity {
        EntityCommands::Players { action } => handle_players(action).await?,
        EntityCommands::Teams { action } => handle_teams(action).await?,
        EntityCommands::Needs { action } => handle_needs(action).await?,
        EntityCommands::Seasons { action } => handle_seasons(action).await?,
        EntityCommands::DraftOrder { action } => handle_draft_order(action).await?,
        EntityCommands::Scouting { action } => handle_scouting(action).await?,
        EntityCommands::Rankings { action } => handle_rankings(action).await?,
        EntityCommands::Freaks { action } => handle_freaks(action).await?,
        EntityCommands::Combine { action } => handle_combine(action).await?,
        EntityCommands::Percentiles { action } => handle_percentiles(action).await?,
        EntityCommands::TheBeast { action } => handle_the_beast(action).await?,
    }

    Ok(())
}

async fn handle_players(action: PlayerActions) -> Result<()> {
    match action {
        PlayerActions::Validate { file } => {
            println!("Validating: {}", file);
            let data = loader::parse_player_file(&file)?;
            println!(
                "Loaded {} players from file (draft year {})",
                data.players.len(),
                data.meta.draft_year
            );

            let result = validator::validate_player_data(&data);
            result.print_summary();

            if !result.valid {
                std::process::exit(1);
            }
        }

        PlayerActions::Load { file, dry_run } => {
            if dry_run {
                println!("DRY RUN - Validating and simulating load: {}", file);
            } else {
                println!("Loading players from: {}", file);
            }

            let data = loader::parse_player_file(&file)?;
            println!(
                "Parsed {} players from file (draft year {})",
                data.players.len(),
                data.meta.draft_year
            );

            // Validate first
            let validation = validator::validate_player_data(&data);
            validation.print_summary();

            if !validation.valid {
                println!("\nAborting load due to validation errors.");
                std::process::exit(1);
            }

            if dry_run {
                // Dry run: simulate loading without database
                let stats = loader::load_players_dry_run(&data)?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            } else {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in environment or .env file");
                let pool = create_pool(&database_url).await?;
                let repo = SqlxPlayerRepository::new(pool);

                let stats = loader::load_players(&data, &repo).await?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }

        PlayerActions::Clear { year } => {
            println!("Clearing all players for draft year {}", year);

            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment or .env file");
            let pool = create_pool(&database_url).await?;
            let repo = SqlxPlayerRepository::new(pool.clone());

            // Count existing players first
            let existing = repo.find_by_draft_year(year).await?;
            let count = existing.len();

            if count == 0 {
                println!("No players found for draft year {}", year);
                return Ok(());
            }

            println!("Found {} players to delete", count);

            // Delete using raw SQL for efficiency
            let result = sqlx::query("DELETE FROM players WHERE draft_year = $1")
                .bind(year)
                .execute(&pool)
                .await?;

            println!("Deleted {} players", result.rows_affected());
        }
    }

    Ok(())
}

async fn handle_teams(action: TeamActions) -> Result<()> {
    match action {
        TeamActions::Validate { file } => {
            println!("Validating: {}", file);
            let data = team_loader::parse_team_file(&file)?;
            println!("Loaded {} teams from file", data.teams.len());

            let result = team_validator::validate_team_data(&data);
            result.print_summary();

            if !result.valid {
                std::process::exit(1);
            }
        }

        TeamActions::Load { file, dry_run } => {
            if dry_run {
                println!("DRY RUN - Validating and simulating load: {}", file);
            } else {
                println!("Loading teams from: {}", file);
            }

            let data = team_loader::parse_team_file(&file)?;
            println!("Parsed {} teams from file", data.teams.len());

            // Validate first
            let validation = team_validator::validate_team_data(&data);
            validation.print_summary();

            if !validation.valid {
                println!("\nAborting load due to validation errors.");
                std::process::exit(1);
            }

            if dry_run {
                // Dry run: simulate loading without database
                let stats = team_loader::load_teams_dry_run(&data)?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            } else {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in environment or .env file");
                let pool = create_pool(&database_url).await?;
                let repo = SqlxTeamRepository::new(pool);

                let stats = team_loader::load_teams(&data, &repo).await?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }

        TeamActions::Clear => {
            println!("Clearing all teams from the database");

            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment or .env file");
            let pool = create_pool(&database_url).await?;

            // Count existing teams first
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM teams")
                .fetch_one(&pool)
                .await?;

            if count == 0 {
                println!("No teams found in database");
                return Ok(());
            }

            println!("Found {} teams to delete", count);

            // Delete all teams
            let result = sqlx::query("DELETE FROM teams").execute(&pool).await?;

            println!("Deleted {} teams", result.rows_affected());
        }
    }

    Ok(())
}

async fn handle_needs(action: NeedActions) -> Result<()> {
    match action {
        NeedActions::Validate { file } => {
            println!("Validating: {}", file);
            let data = team_need_loader::parse_team_need_file(&file)?;
            println!("Loaded {} team entries from file", data.team_needs.len());

            let result = team_need_validator::validate_team_need_data(&data);
            result.print_summary();

            if !result.valid {
                std::process::exit(1);
            }
        }

        NeedActions::Load { file, dry_run } => {
            if dry_run {
                println!("DRY RUN - Validating and simulating load: {}", file);
            } else {
                println!("Loading team needs from: {}", file);
            }

            let data = team_need_loader::parse_team_need_file(&file)?;
            println!("Parsed {} team entries from file", data.team_needs.len());

            // Validate first
            let validation = team_need_validator::validate_team_need_data(&data);
            validation.print_summary();

            if !validation.valid {
                println!("\nAborting load due to validation errors.");
                std::process::exit(1);
            }

            if dry_run {
                // Dry run: simulate loading without database
                let stats = team_need_loader::load_team_needs_dry_run(&data)?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            } else {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in environment or .env file");
                let pool = create_pool(&database_url).await?;
                let team_repo = SqlxTeamRepository::new(pool.clone());
                let team_need_repo = SqlxTeamNeedRepository::new(pool);

                let stats =
                    team_need_loader::load_team_needs(&data, &team_repo, &team_need_repo).await?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }

        NeedActions::Clear => {
            println!("Clearing all team needs from the database");

            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment or .env file");
            let pool = create_pool(&database_url).await?;

            // Count existing team needs first
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM team_needs")
                .fetch_one(&pool)
                .await?;

            if count == 0 {
                println!("No team needs found in database");
                return Ok(());
            }

            println!("Found {} team needs to delete", count);

            // Delete all team needs
            let result = sqlx::query("DELETE FROM team_needs").execute(&pool).await?;

            println!("Deleted {} team needs", result.rows_affected());
        }
    }

    Ok(())
}

async fn handle_seasons(action: SeasonActions) -> Result<()> {
    match action {
        SeasonActions::Validate { file } => {
            println!("Validating: {}", file);
            let data = team_season_loader::parse_team_season_file(&file)?;
            println!(
                "Loaded {} team seasons from file (season year {})",
                data.team_seasons.len(),
                data.meta.season_year
            );

            let result = team_season_validator::validate_team_season_data(&data);
            result.print_summary();

            if !result.valid {
                std::process::exit(1);
            }
        }

        SeasonActions::Load { file, dry_run } => {
            if dry_run {
                println!("DRY RUN - Validating and simulating load: {}", file);
            } else {
                println!("Loading team seasons from: {}", file);
            }

            let data = team_season_loader::parse_team_season_file(&file)?;
            println!(
                "Parsed {} team seasons from file (season year {})",
                data.team_seasons.len(),
                data.meta.season_year
            );

            // Validate first
            let validation = team_season_validator::validate_team_season_data(&data);
            validation.print_summary();

            if !validation.valid {
                println!("\nAborting load due to validation errors.");
                std::process::exit(1);
            }

            if dry_run {
                // Dry run: simulate loading without database
                let stats = team_season_loader::load_team_seasons_dry_run(&data)?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            } else {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in environment or .env file");
                let pool = create_pool(&database_url).await?;
                let team_repo = SqlxTeamRepository::new(pool.clone());
                let team_season_repo = SqlxTeamSeasonRepository::new(pool);

                let stats =
                    team_season_loader::load_team_seasons(&data, &team_repo, &team_season_repo)
                        .await?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }

        SeasonActions::Clear { year } => {
            println!("Clearing all team seasons for year {}", year);

            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment or .env file");
            let pool = create_pool(&database_url).await?;

            // Count existing team seasons first
            let count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM team_seasons WHERE season_year = $1")
                    .bind(year)
                    .fetch_one(&pool)
                    .await?;

            if count == 0 {
                println!("No team seasons found for year {}", year);
                return Ok(());
            }

            println!("Found {} team seasons to delete", count);

            // Delete team seasons for the year
            let result = sqlx::query("DELETE FROM team_seasons WHERE season_year = $1")
                .bind(year)
                .execute(&pool)
                .await?;

            println!("Deleted {} team seasons", result.rows_affected());
        }
    }

    Ok(())
}

async fn handle_draft_order(action: DraftOrderActions) -> Result<()> {
    match action {
        DraftOrderActions::Validate { file } => {
            println!("Validating: {}", file);
            let data = draft_order_loader::parse_draft_order_file(&file)?;
            println!(
                "Loaded {} picks from file (draft year {})",
                data.draft_order.len(),
                data.meta.draft_year
            );

            let result = draft_order_validator::validate_draft_order_data(&data);
            result.print_summary();

            if !result.valid {
                std::process::exit(1);
            }
        }

        DraftOrderActions::Load { file, dry_run } => {
            if dry_run {
                println!("DRY RUN - Validating and simulating load: {}", file);
            } else {
                println!("Loading draft order from: {}", file);
            }

            let data = draft_order_loader::parse_draft_order_file(&file)?;
            println!(
                "Parsed {} picks from file (draft year {})",
                data.draft_order.len(),
                data.meta.draft_year
            );

            // Validate first
            let validation = draft_order_validator::validate_draft_order_data(&data);
            validation.print_summary();

            if !validation.valid {
                println!("\nAborting load due to validation errors.");
                std::process::exit(1);
            }

            if dry_run {
                let stats = draft_order_loader::load_draft_order_dry_run(&data)?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            } else {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in environment or .env file");
                let pool = create_pool(&database_url).await?;
                let team_repo = SqlxTeamRepository::new(pool.clone());
                let draft_repo = SqlxDraftRepository::new(pool.clone());
                let pick_repo = SqlxDraftPickRepository::new(pool);

                let stats = draft_order_loader::load_draft_order(
                    &data,
                    &team_repo,
                    &draft_repo,
                    &pick_repo,
                )
                .await?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }

        DraftOrderActions::Clear { year } => {
            println!("Clearing draft order for year {}", year);

            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment or .env file");
            let pool = create_pool(&database_url).await?;

            // Find draft by year
            let draft =
                sqlx::query_scalar::<_, sqlx::types::Uuid>("SELECT id FROM drafts WHERE year = $1")
                    .bind(year)
                    .fetch_optional(&pool)
                    .await?;

            match draft {
                Some(draft_id) => {
                    // Count picks
                    let pick_count: i64 =
                        sqlx::query_scalar("SELECT COUNT(*) FROM draft_picks WHERE draft_id = $1")
                            .bind(draft_id)
                            .fetch_one(&pool)
                            .await?;

                    if pick_count == 0 {
                        println!("No draft picks found for year {}", year);
                        return Ok(());
                    }

                    println!("Found {} draft picks to delete", pick_count);

                    // Delete picks
                    let result = sqlx::query("DELETE FROM draft_picks WHERE draft_id = $1")
                        .bind(draft_id)
                        .execute(&pool)
                        .await?;

                    println!("Deleted {} draft picks", result.rows_affected());
                }
                None => {
                    println!("No draft found for year {}", year);
                }
            }
        }
    }

    Ok(())
}

async fn handle_scouting(action: ScoutingActions) -> Result<()> {
    match action {
        ScoutingActions::Validate { file } => {
            println!("Validating: {}", file);
            let data = scouting_report_loader::parse_ranking_file(&file)?;
            println!(
                "Loaded {} rankings from file (draft year {}, source: {})",
                data.rankings.len(),
                data.meta.draft_year,
                data.meta.source
            );

            let result = scouting_report_validator::validate_ranking_data(&data);
            result.print_summary();

            if !result.valid {
                std::process::exit(1);
            }
        }

        ScoutingActions::Load { file, dry_run } => {
            if dry_run {
                println!("DRY RUN - Validating and simulating load: {}", file);
            } else {
                println!("Loading scouting reports from rankings: {}", file);
            }

            let data = scouting_report_loader::parse_ranking_file(&file)?;
            println!(
                "Parsed {} rankings from file (draft year {}, source: {})",
                data.rankings.len(),
                data.meta.draft_year,
                data.meta.source
            );

            // Validate first
            let validation = scouting_report_validator::validate_ranking_data(&data);
            validation.print_summary();

            if !validation.valid {
                println!("\nAborting load due to validation errors.");
                std::process::exit(1);
            }

            if dry_run {
                let stats = scouting_report_loader::load_scouting_reports_dry_run(&data)?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            } else {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in environment or .env file");
                let pool = create_pool(&database_url).await?;
                let player_repo = SqlxPlayerRepository::new(pool.clone());
                let team_repo = SqlxTeamRepository::new(pool.clone());

                let stats = scouting_report_loader::load_scouting_reports(
                    &data,
                    &player_repo,
                    &team_repo,
                    &pool,
                )
                .await?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }

        ScoutingActions::Clear { year } => {
            println!("Clearing all scouting reports for draft year {}", year);

            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment or .env file");
            let pool = create_pool(&database_url).await?;

            // Count existing scouting reports for this draft year
            let count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM scouting_reports WHERE player_id IN (SELECT id FROM players WHERE draft_year = $1)"
            )
            .bind(year)
            .fetch_one(&pool)
            .await?;

            if count == 0 {
                println!("No scouting reports found for draft year {}", year);
                return Ok(());
            }

            println!("Found {} scouting reports to delete", count);

            let result = sqlx::query(
                "DELETE FROM scouting_reports WHERE player_id IN (SELECT id FROM players WHERE draft_year = $1)"
            )
            .bind(year)
            .execute(&pool)
            .await?;

            println!("Deleted {} scouting reports", result.rows_affected());
        }
    }

    Ok(())
}

async fn handle_rankings(action: RankingsActions) -> Result<()> {
    match action {
        RankingsActions::Validate { file } => {
            println!("Validating: {}", file);
            let data = scouting_report_loader::parse_ranking_file(&file)?;
            println!(
                "Loaded {} rankings from file (draft year {}, source: {})",
                data.rankings.len(),
                data.meta.draft_year,
                data.meta.source
            );

            let result = rankings_validator::validate_ranking_data(&data);
            result.print_summary();

            if !result.valid {
                std::process::exit(1);
            }
        }

        RankingsActions::Load { file, dry_run } => {
            if dry_run {
                println!("DRY RUN - Validating and simulating load: {}", file);
            } else {
                println!("Loading prospect rankings from: {}", file);
            }

            let data = scouting_report_loader::parse_ranking_file(&file)?;
            println!(
                "Parsed {} rankings from file (draft year {}, source: {})",
                data.rankings.len(),
                data.meta.draft_year,
                data.meta.source
            );

            // Validate first
            let validation = rankings_validator::validate_ranking_data(&data);
            validation.print_summary();

            if !validation.valid {
                println!("\nAborting load due to validation errors.");
                std::process::exit(1);
            }

            if dry_run {
                let stats = rankings_loader::load_rankings_dry_run(&data)?;
                stats.print_summary();
            } else {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in environment or .env file");
                let pool = create_pool(&database_url).await?;
                let player_repo = SqlxPlayerRepository::new(pool.clone());
                let team_repo = SqlxTeamRepository::new(pool.clone());
                let ranking_source_repo = SqlxRankingSourceRepository::new(pool.clone());
                let scouting_report_repo = SqlxScoutingReportRepository::new(pool.clone());

                let stats = rankings_loader::load_rankings(
                    &data,
                    &pool,
                    &player_repo,
                    &team_repo,
                    &ranking_source_repo,
                    &scouting_report_repo,
                )
                .await?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }

        RankingsActions::Clear { source } => {
            println!("Clearing rankings for source: {}", source);

            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment or .env file");
            let pool = create_pool(&database_url).await?;
            let ranking_source_repo = SqlxRankingSourceRepository::new(pool.clone());
            let prospect_ranking_repo = SqlxProspectRankingRepository::new(pool.clone());

            let deleted = rankings_loader::clear_rankings(
                &source,
                &ranking_source_repo,
                &prospect_ranking_repo,
            )
            .await?;
            println!("Deleted {} rankings", deleted);
        }
    }

    Ok(())
}

async fn handle_freaks(action: FreaksActions) -> Result<()> {
    match action {
        FreaksActions::Validate { file } => {
            println!("Validating: {}", file);
            let data = feldman_freak_loader::parse_freaks_file(&file)?;
            println!(
                "Loaded {} freaks from file (year {})",
                data.freaks.len(),
                data.meta.year
            );

            let result = feldman_freak_validator::validate_freaks_data(&data);
            result.print_summary();

            if !result.valid {
                std::process::exit(1);
            }
        }

        FreaksActions::Load { file, dry_run } => {
            if dry_run {
                println!("DRY RUN - Validating and simulating load: {}", file);
            } else {
                println!("Loading Feldman Freaks from: {}", file);
            }

            let data = feldman_freak_loader::parse_freaks_file(&file)?;
            println!(
                "Parsed {} freaks from file (year {})",
                data.freaks.len(),
                data.meta.year
            );

            // Validate first
            let validation = feldman_freak_validator::validate_freaks_data(&data);
            validation.print_summary();

            if !validation.valid {
                println!("\nAborting load due to validation errors.");
                std::process::exit(1);
            }

            if dry_run {
                let stats = feldman_freak_loader::load_freaks_dry_run(&data)?;
                stats.print_summary();
            } else {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in environment or .env file");
                let pool = create_pool(&database_url).await?;
                let player_repo = SqlxPlayerRepository::new(pool.clone());
                let freak_repo = SqlxFeldmanFreakRepository::new(pool);

                let stats =
                    feldman_freak_loader::load_freaks(&data, &player_repo, &freak_repo).await?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }

        FreaksActions::Clear { year } => {
            println!("Clearing Feldman Freaks for year {}", year);

            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment or .env file");
            let pool = create_pool(&database_url).await?;
            let freak_repo = SqlxFeldmanFreakRepository::new(pool);

            let deleted = feldman_freak_loader::clear_freaks(year, &freak_repo).await?;
            println!("Deleted {} freaks", deleted);
        }
    }

    Ok(())
}

async fn handle_combine(action: CombineActions) -> Result<()> {
    match action {
        CombineActions::Validate { file } => {
            println!("Validating: {}", file);
            let data = combine_loader::parse_combine_file(&file)?;
            println!(
                "Loaded {} combine entries from file (year {}, source: {})",
                data.combine_results.len(),
                data.meta.year,
                data.meta.source
            );

            // Validate year is reasonable
            let mut errors: Vec<String> = Vec::new();
            let mut warnings: Vec<String> = Vec::new();

            if data.meta.year < 2020 || data.meta.year > 2030 {
                errors.push(format!(
                    "Year {} is outside reasonable range (2020-2030)",
                    data.meta.year
                ));
            }

            if data.meta.source.trim().is_empty() {
                errors.push("Meta source is empty".to_string());
            }

            let mut entries_with_measurements = 0;
            let mut entries_without_measurements = 0;
            let mut empty_source_count = 0;
            let mut invalid_source_count = 0;

            for (i, entry) in data.combine_results.iter().enumerate() {
                if entry.source.trim().is_empty() {
                    errors.push(format!(
                        "Entry {} ({} {}): source is empty",
                        i, entry.first_name, entry.last_name
                    ));
                    empty_source_count += 1;
                } else if entry
                    .source
                    .parse::<domain::models::CombineSource>()
                    .is_err()
                {
                    errors.push(format!(
                        "Entry {} ({} {}): invalid source '{}'",
                        i, entry.first_name, entry.last_name, entry.source
                    ));
                    invalid_source_count += 1;
                }

                if combine_loader::entry_has_any_measurement(entry) {
                    entries_with_measurements += 1;
                } else {
                    entries_without_measurements += 1;
                }
            }

            if entries_with_measurements == 0 && !data.combine_results.is_empty() {
                warnings.push("No entries have any measurements".to_string());
            }

            println!("\nValidation Summary:");
            println!(
                "  Total entries:              {}",
                data.combine_results.len()
            );
            println!(
                "  With measurements:          {}",
                entries_with_measurements
            );
            println!(
                "  Without measurements:       {}",
                entries_without_measurements
            );
            if empty_source_count > 0 {
                println!("  Empty source strings:       {}", empty_source_count);
            }
            if invalid_source_count > 0 {
                println!("  Invalid source strings:     {}", invalid_source_count);
            }

            if !warnings.is_empty() {
                println!("\n  Warnings: {}", warnings.len());
                for w in &warnings {
                    println!("    - {}", w);
                }
            }

            if !errors.is_empty() {
                println!("\n  Errors: {}", errors.len());
                for e in &errors {
                    println!("    - {}", e);
                }
                std::process::exit(1);
            } else {
                println!("\n  Result: VALID");
            }
        }

        CombineActions::Load { file, dry_run } => {
            if dry_run {
                println!("DRY RUN - Loading combine data: {}", file);
            } else {
                println!("Loading combine data from: {}", file);
            }

            let data = combine_loader::parse_combine_file(&file)?;
            println!(
                "Parsed {} combine entries from file (year {}, source: {})",
                data.combine_results.len(),
                data.meta.year,
                data.meta.source
            );

            if dry_run {
                let stats = combine_loader::load_combine_data_dry_run(&data)?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            } else {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in environment or .env file");
                let pool = create_pool(&database_url).await?;
                let player_repo = SqlxPlayerRepository::new(pool.clone());
                let combine_repo = SqlxCombineResultsRepository::new(pool);

                let stats =
                    combine_loader::load_combine_data(&data, &player_repo, &combine_repo).await?;
                stats.print_summary();

                if !stats.errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }

        CombineActions::Clear { year } => {
            println!("Clearing combine results for year {}", year);

            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment or .env file");
            let pool = create_pool(&database_url).await?;

            let result = sqlx::query("DELETE FROM combine_results WHERE year = $1")
                .bind(year)
                .execute(&pool)
                .await?;

            println!("Deleted {} combine results", result.rows_affected());
        }
    }

    Ok(())
}

async fn handle_percentiles(action: PercentilesActions) -> Result<()> {
    match action {
        PercentilesActions::Validate { file } => {
            println!("Validating: {}", file);
            let data = percentile_loader::parse_percentile_file(&file)?;
            println!(
                "Loaded {} percentile entries from file (source: {})",
                data.percentiles.len(),
                data.meta.source
            );

            let mut positions = std::collections::HashSet::new();
            let mut measurements = std::collections::HashSet::new();
            for entry in &data.percentiles {
                positions.insert(entry.position.clone());
                measurements.insert(entry.measurement.clone());
            }

            println!("\nSummary:");
            println!("  Positions:    {} unique", positions.len());
            println!("  Measurements: {} unique", measurements.len());
            println!("  Total entries: {}", data.percentiles.len());
            println!("\n  Result: VALID");
        }

        PercentilesActions::Load { file } => {
            println!("Loading combine percentiles from: {}", file);

            let data = percentile_loader::parse_percentile_file(&file)?;
            println!(
                "Parsed {} percentile entries from file (source: {})",
                data.percentiles.len(),
                data.meta.source
            );

            let database_url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set in environment or .env file");
            let pool = create_pool(&database_url).await?;
            let repo = SqlxCombinePercentileRepository::new(pool);

            let stats = percentile_loader::load_percentiles(&data, &repo).await?;

            println!("\nLoad Results:");
            println!("  Upserted: {}", stats.upserted);
            if !stats.errors.is_empty() {
                println!("  Errors: {}", stats.errors.len());
                for e in &stats.errors {
                    println!("    - {}", e);
                }
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

async fn handle_the_beast(action: TheBeastActions) -> Result<()> {
    match action {
        TheBeastActions::Load { file, dry_run } => {
            if dry_run {
                println!("DRY RUN - Validating and simulating load: {}", file);
            } else {
                println!("Loading The Beast 2026 from: {}", file);
            }

            let data = the_beast_loader::parse_beast_file(&file)?;
            println!(
                "Parsed {} prospects from file (draft year {}, source: {})",
                data.prospects.len(),
                data.meta.draft_year,
                data.meta.source
            );

            if dry_run {
                let stats = the_beast_loader::load_beast_dry_run(&data)?;
                stats.print_summary();
            } else {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set in environment or .env file");
                let pool = create_pool(&database_url).await?;
                let player_repo = SqlxPlayerRepository::new(pool.clone());
                let profile_repo = SqlxProspectProfileRepository::new(pool.clone());
                let combine_repo = SqlxCombineResultsRepository::new(pool.clone());
                let ranking_source_repo = SqlxRankingSourceRepository::new(pool.clone());
                let prospect_ranking_repo = SqlxProspectRankingRepository::new(pool.clone());

                let stats = the_beast_loader::load_beast(
                    &data,
                    &pool,
                    &player_repo,
                    &profile_repo,
                    &combine_repo,
                    &ranking_source_repo,
                    &prospect_ranking_repo,
                )
                .await?;
                stats.print_summary();

                if !stats.errors.is_empty() && stats.profiles_upserted == 0 {
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
