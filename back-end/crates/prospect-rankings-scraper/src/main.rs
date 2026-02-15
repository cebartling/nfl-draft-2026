mod merge;
mod models;
mod scrapers;
mod template;

use std::path::Path;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "prospect-rankings-scraper")]
#[command(about = "Scrape NFL prospect rankings from draft sites and generate JSON files")]
struct Cli {
    /// Source to scrape from
    #[arg(short, long, default_value = "tankathon")]
    source: String,

    /// Draft year to scrape
    #[arg(short, long, default_value = "2026")]
    year: i32,

    /// Output file path
    #[arg(short, long, default_value = "data/rankings/rankings.json")]
    output: String,

    /// Maximum number of prospects to scrape (for paginated sources)
    #[arg(short, long, default_value = "200")]
    max_prospects: usize,

    /// Generate template without scraping (useful when site is unavailable)
    #[arg(long)]
    template: bool,

    /// Use Playwright browser automation instead of HTTP scraping.
    /// Requires Node.js and Playwright to be installed.
    /// Handles JS-rendered SPAs that reqwest cannot parse.
    #[arg(long)]
    browser: bool,

    /// Merge multiple ranking files instead of scraping.
    /// Requires --primary and at least one --secondary.
    #[arg(long)]
    merge: bool,

    /// Primary ranking file for merge (provides base rankings)
    #[arg(long)]
    primary: Option<String>,

    /// Secondary ranking file(s) for merge (unique prospects appended)
    #[arg(long)]
    secondary: Vec<String>,

    /// Allow template fallback to overwrite existing non-template data
    #[arg(long)]
    allow_template_fallback: bool,
}

/// Minimal struct for reading just the meta.source field from an existing file.
/// Uses Option<String> so it works even if the field is absent (older files).
#[derive(serde::Deserialize)]
struct ExistingMeta {
    source: Option<String>,
}

#[derive(serde::Deserialize)]
struct ExistingFile {
    meta: ExistingMeta,
}

/// Check if the existing output file contains non-template (curated/scraped) data.
/// Returns true if the file exists and its meta.source is NOT "template"
/// (or if the source field is absent, which implies real data).
fn existing_file_has_real_data(path: &str) -> bool {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(data) = serde_json::from_str::<ExistingFile>(&contents) else {
        // If we can't parse it, treat it as real data to be safe
        return true;
    };
    data.meta.source.as_deref() != Some("template")
}

/// Check if the generated data is template-based (either explicitly requested
/// or produced by a scraping fallback).
fn is_template_data(data: &models::RankingData) -> bool {
    data.meta.source == "template"
}

/// Write an RFC 3339 timestamp file alongside the output to track when
/// rankings were last successfully scraped/generated.
fn write_timestamp_file(output_path: &str) -> Result<()> {
    let parent = Path::new(output_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let timestamp_path = parent.join(".rankings_last_scraped");
    let now = chrono::Utc::now().to_rfc3339();
    std::fs::write(&timestamp_path, &now)?;
    println!("Wrote timestamp to: {}", timestamp_path.display());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.merge {
        return run_merge(&cli);
    }

    println!("NFL Prospect Rankings Scraper");
    println!("Source: {}", cli.source);
    println!("Year: {}", cli.year);
    println!("Output: {}", cli.output);

    let data = if cli.template {
        println!("\nGenerating template rankings...");
        template::generate_template(cli.year)
    } else if cli.browser {
        println!("\nUsing Playwright browser automation...");
        match scrapers::tankathon::scrape_with_browser(cli.year, &cli.output) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Browser scraping failed: {}", e);
                println!("Generating template instead...");
                template::generate_template(cli.year)
            }
        }
    } else {
        match cli.source.to_lowercase().as_str() {
            "tankathon" => {
                println!("\nFetching prospect rankings from Tankathon.com...");
                match scrapers::tankathon::fetch_html(cli.year).await {
                    Ok(html) => {
                        println!("Fetched {} bytes of HTML", html.len());
                        match scrapers::tankathon::parse_html(&html, cli.year) {
                            Ok(data) if data.rankings.is_empty() => {
                                println!(
                                    "\nScraping produced no results. Generating template instead."
                                );
                                println!(
                                    "Edit the output file manually to match real prospect rankings."
                                );
                                println!("Tip: Use --browser flag for JS-rendered sites.");
                                template::generate_template(cli.year)
                            }
                            Ok(data) => data,
                            Err(e) => {
                                eprintln!("Failed to parse HTML: {}", e);
                                println!("Generating template instead...");
                                template::generate_template(cli.year)
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch Tankathon: {}", e);
                        println!("Generating template instead...");
                        template::generate_template(cli.year)
                    }
                }
            }
            "drafttek" => {
                println!(
                    "\nFetching prospect rankings from DraftTek.com (max {} prospects)...",
                    cli.max_prospects
                );
                match scrapers::drafttek::fetch_rankings(cli.year, cli.max_prospects).await {
                    Ok(data) if data.rankings.is_empty() => {
                        println!("\nScraping produced no results. Generating template instead.");
                        println!("Edit the output file manually to match real prospect rankings.");
                        template::generate_template(cli.year)
                    }
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Failed to fetch DraftTek: {}", e);
                        println!("Generating template instead...");
                        template::generate_template(cli.year)
                    }
                }
            }
            "walterfootball" => {
                println!("\nFetching prospect rankings from WalterFootball.com...");
                match scrapers::walterfootball::fetch_html(cli.year).await {
                    Ok(html) => {
                        println!("Fetched {} bytes of HTML", html.len());
                        match scrapers::walterfootball::parse_html(&html, cli.year) {
                            Ok(data) if data.rankings.is_empty() => {
                                println!(
                                    "\nScraping produced no results. Generating template instead."
                                );
                                template::generate_template(cli.year)
                            }
                            Ok(data) => data,
                            Err(e) => {
                                eprintln!("Failed to parse HTML: {}", e);
                                println!("Generating template instead...");
                                template::generate_template(cli.year)
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch Walter Football: {}", e);
                        println!("Generating template instead...");
                        template::generate_template(cli.year)
                    }
                }
            }
            other => {
                eprintln!(
                    "Unknown source '{}'. Supported sources: tankathon, drafttek, walterfootball",
                    other
                );
                eprintln!("Generating template instead...");
                template::generate_template(cli.year)
            }
        }
    };

    // Safety guard: refuse to overwrite real data with template output
    if is_template_data(&data) && !cli.template && !cli.allow_template_fallback {
        if existing_file_has_real_data(&cli.output) {
            eprintln!(
                "\nERROR: Scraping failed and would fall back to template data, but '{}' \
                 already contains real (non-template) data.",
                cli.output
            );
            eprintln!("Refusing to overwrite to protect your curated rankings.");
            eprintln!("To overwrite anyway, pass --allow-template-fallback");
            std::process::exit(1);
        }
    }

    println!("\nRankings summary:");
    println!("  Source: {}", data.meta.source);
    println!("  Year: {}", data.meta.draft_year);
    println!("  Total prospects: {}", data.meta.total_prospects);

    if !data.rankings.is_empty() {
        println!("\n  Top 10:");
        for entry in data.rankings.iter().take(10) {
            println!(
                "    {}. {} {} - {} ({})",
                entry.rank, entry.first_name, entry.last_name, entry.position, entry.school
            );
        }
    }

    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(&cli.output).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write JSON output (skip if --browser already wrote it)
    if !cli.browser {
        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(&cli.output, &json)?;
        println!("\nWrote rankings to: {}", cli.output);
    } else {
        println!("\nRankings written by browser scraper to: {}", cli.output);
    }

    // Write timestamp file for staleness tracking
    write_timestamp_file(&cli.output)?;

    Ok(())
}

fn run_merge(cli: &Cli) -> Result<()> {
    let primary_path = cli
        .primary
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--merge requires --primary <file>"))?;

    if cli.secondary.is_empty() {
        anyhow::bail!("--merge requires at least one --secondary <file>");
    }

    println!("NFL Prospect Rankings Merger");
    println!("Primary: {}", primary_path);
    for (i, s) in cli.secondary.iter().enumerate() {
        println!("Secondary {}: {}", i + 1, s);
    }
    println!("Output: {}", cli.output);

    let primary = merge::load_ranking_file(primary_path)?;
    println!(
        "\nLoaded primary: {} prospects from {}",
        primary.rankings.len(),
        primary.meta.source
    );

    let mut secondaries = Vec::new();
    for path in &cli.secondary {
        let data = merge::load_ranking_file(path)?;
        println!(
            "Loaded secondary: {} prospects from {}",
            data.rankings.len(),
            data.meta.source
        );
        secondaries.push(data);
    }

    let merged = merge::merge_rankings(primary, secondaries)?;

    println!("\nMerge result:");
    println!("  Total unique prospects: {}", merged.rankings.len());

    if !merged.rankings.is_empty() {
        println!("\n  Top 10:");
        for entry in merged.rankings.iter().take(10) {
            println!(
                "    {}. {} {} - {} ({})",
                entry.rank, entry.first_name, entry.last_name, entry.position, entry.school
            );
        }
    }

    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(&cli.output).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&merged)?;
    std::fs::write(&cli.output, &json)?;
    println!("\nWrote merged rankings to: {}", cli.output);

    // Write timestamp file for staleness tracking
    write_timestamp_file(&cli.output)?;

    Ok(())
}
