mod mock_generator;
mod percentile_template;

use std::path::Path;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "combine-data-scraper")]
#[command(about = "Generate combine percentile baselines and mock combine data")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate template percentile data from known NFL averages
    Template {
        /// Output file path
        #[arg(short, long, default_value = "data/combine_percentiles.json")]
        output: String,
    },
    /// Generate mock combine data for prospects
    MockData {
        /// Input prospects JSON file
        #[arg(short, long, default_value = "data/players_2026.json")]
        input: String,

        /// Output file path
        #[arg(short, long, default_value = "data/combine_2026_mock.json")]
        output: String,

        /// Draft year
        #[arg(short, long, default_value = "2026")]
        year: i32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Template { output } => {
            println!("Generating template percentile data...");
            let data = percentile_template::generate_template_percentiles();

            let json = serde_json::to_string_pretty(&data)?;

            if let Some(parent) = Path::new(&output).parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&output, json)?;

            println!(
                "Generated {} percentile entries to {}",
                data.percentiles.len(),
                output
            );
        }
        Commands::MockData {
            input,
            output,
            year,
        } => {
            println!("Generating mock combine data from {}...", input);

            let contents = std::fs::read_to_string(&input)?;
            let prospect_data: mock_generator::ProspectData = serde_json::from_str(&contents)?;

            let data =
                mock_generator::generate_mock_combine_data(&prospect_data.players, year);

            let json = serde_json::to_string_pretty(&data)?;

            if let Some(parent) = Path::new(&output).parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&output, json)?;

            println!(
                "Generated {} combine entries for {} players to {}",
                data.meta.entry_count, data.meta.player_count, output
            );
        }
    }

    Ok(())
}
