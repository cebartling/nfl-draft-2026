mod loader;
mod position_mapper;
mod validator;

use anyhow::Result;
use clap::{Parser, Subcommand};
use db::{create_pool, repositories::SqlxPlayerRepository};
use domain::repositories::PlayerRepository;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "seed-players")]
#[command(about = "Seed NFL Draft player data into the database")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { file } => {
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

        Commands::Load { file, dry_run } => {
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

        Commands::Clear { year } => {
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
