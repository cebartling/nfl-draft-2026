mod merge;
mod mock_generator;
mod models;
mod percentile_template;
mod scrapers;

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "combine-data-scraper")]
#[command(about = "Generate combine percentile baselines, mock combine data, and scrape real combine results")]
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
    /// Scrape real combine data from web sources
    Scrape {
        /// Source to scrape: pfr, mockdraftable
        #[arg(short, long, default_value = "pfr")]
        source: String,

        /// Draft year
        #[arg(short = 'y', long, default_value = "2026")]
        year: i32,

        /// Output file path
        #[arg(short, long, default_value = "data/combine_2026.json")]
        output: String,

        /// Use Playwright browser fallback
        #[arg(long)]
        browser: bool,

        /// Merge PFR + Mockdraftable sources
        #[arg(long)]
        merge: bool,

        /// Allow template fallback if scraping fails
        #[arg(long)]
        allow_template_fallback: bool,
    },
}

fn write_json(data: &impl serde::Serialize, output: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    if let Some(parent) = Path::new(output).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, format!("{}\n", json))?;
    Ok(())
}

fn write_timestamp() -> Result<()> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    std::fs::write(".combine_last_scraped", &timestamp)?;
    Ok(())
}

/// Shell out to Playwright script for browser-based scraping.
fn scrape_with_browser(year: i32, output: &str, source: &str) -> Result<models::CombineData> {
    let script_path = Path::new("scripts/scrape-combine.mjs");
    if !script_path.exists() {
        anyhow::bail!(
            "Browser scraping script not found at {}. Run without --browser.",
            script_path.display()
        );
    }

    eprintln!("Using Playwright browser for {} scraping...", source);

    let status = Command::new("node")
        .arg(script_path)
        .arg("--year")
        .arg(year.to_string())
        .arg("--output")
        .arg(output)
        .arg("--source")
        .arg(source)
        .status()
        .context("Failed to run Playwright scraping script")?;

    if !status.success() {
        anyhow::bail!("Playwright scraping script failed with status: {}", status);
    }

    let contents = std::fs::read_to_string(output)
        .context("Failed to read Playwright output")?;
    let data: models::CombineData = serde_json::from_str(&contents)
        .context("Failed to parse Playwright output JSON")?;

    Ok(data)
}

async fn scrape_source(
    source: &str,
    year: i32,
    output: &str,
    browser: bool,
) -> Result<models::CombineData> {
    if browser {
        return scrape_with_browser(year, output, source);
    }

    match source {
        "pfr" => scrapers::pfr::scrape(year).await,
        "mockdraftable" => scrapers::mockdraftable::scrape(year).await,
        _ => anyhow::bail!("Unknown source: {}. Use 'pfr' or 'mockdraftable'.", source),
    }
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
        Commands::Scrape {
            source,
            year,
            output,
            browser,
            merge: do_merge,
            allow_template_fallback,
        } => {
            if do_merge {
                // Scrape both sources, merge them
                let output_path = Path::new(&output);
                let stem = output_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("combine");
                let parent = output_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."));
                let pfr_output = parent
                    .join(format!("{}_pfr.json", stem))
                    .to_string_lossy()
                    .to_string();
                let md_output = parent
                    .join(format!("{}_mockdraftable.json", stem))
                    .to_string_lossy()
                    .to_string();

                let pfr_result = scrape_source("pfr", year, &pfr_output, browser).await;
                let md_result =
                    scrape_source("mockdraftable", year, &md_output, browser).await;

                match (pfr_result, md_result) {
                    (Ok(pfr_data), Ok(md_data)) => {
                        // Save individual files
                        write_json(&pfr_data, &pfr_output)?;
                        println!("Wrote PFR data: {} entries to {}", pfr_data.combine_results.len(), pfr_output);

                        write_json(&md_data, &md_output)?;
                        println!("Wrote Mockdraftable data: {} entries to {}", md_data.combine_results.len(), md_output);

                        // Merge
                        let merged = merge::merge_combine_data(pfr_data, vec![md_data])?;
                        write_json(&merged, &output)?;
                        println!(
                            "Merged: {} entries to {}",
                            merged.combine_results.len(),
                            output
                        );
                    }
                    (Ok(pfr_data), Err(md_err)) => {
                        eprintln!("WARNING: Mockdraftable scrape failed: {}", md_err);
                        write_json(&pfr_data, &pfr_output)?;
                        write_json(&pfr_data, &output)?;
                        println!(
                            "Using PFR only: {} entries to {}",
                            pfr_data.combine_results.len(),
                            output
                        );
                    }
                    (Err(pfr_err), Ok(md_data)) => {
                        eprintln!("WARNING: PFR scrape failed: {}", pfr_err);
                        write_json(&md_data, &md_output)?;
                        write_json(&md_data, &output)?;
                        println!(
                            "Using Mockdraftable only: {} entries to {}",
                            md_data.combine_results.len(),
                            output
                        );
                    }
                    (Err(pfr_err), Err(md_err)) => {
                        eprintln!("ERROR: Both sources failed:");
                        eprintln!("  PFR: {}", pfr_err);
                        eprintln!("  Mockdraftable: {}", md_err);

                        if allow_template_fallback {
                            eprintln!("Falling back to template data...");
                            // Template fallback would go here if needed
                            anyhow::bail!("Template fallback not yet implemented for scrape mode");
                        } else {
                            anyhow::bail!("Both scrapers failed. Use --allow-template-fallback to use mock data.");
                        }
                    }
                }
            } else {
                // Single source
                match scrape_source(&source, year, &output, browser).await {
                    Ok(data) => {
                        write_json(&data, &output)?;
                        println!(
                            "Scraped {} entries from {} to {}",
                            data.combine_results.len(),
                            source,
                            output
                        );
                    }
                    Err(e) => {
                        eprintln!("ERROR: Scrape failed: {}", e);
                        if allow_template_fallback {
                            eprintln!("Falling back to template data...");
                            anyhow::bail!("Template fallback not yet implemented for scrape mode");
                        } else {
                            return Err(e);
                        }
                    }
                }
            }

            write_timestamp()?;
        }
    }

    Ok(())
}
