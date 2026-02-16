mod scraper;
mod team_name_mapper;

use std::path::Path;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "draft-order-scraper")]
#[command(about = "Scrape NFL draft order from Tankathon and generate JSON")]
struct Cli {
    /// Draft year to scrape
    #[arg(short, long, default_value = "2026")]
    year: i32,

    /// Output file path
    #[arg(short, long, default_value = "data/draft_order_2026.json")]
    output: String,

    /// Generate template without scraping (useful when site is unavailable)
    #[arg(long)]
    template: bool,

    /// Allow template fallback to overwrite existing non-template data
    #[arg(long)]
    allow_template_fallback: bool,
}

/// Minimal struct for reading just the meta.source field from an existing file.
/// Uses Option<String> so it works even if the field is absent (older hand-curated files).
#[derive(serde::Deserialize)]
struct ExistingMeta {
    source: Option<String>,
}

#[derive(serde::Deserialize)]
struct ExistingFile {
    meta: ExistingMeta,
}

/// Check if the existing output file contains non-template (curated) data.
/// Returns true if the file exists and its meta.source is NOT "template"
/// (or if the source field is absent, which implies hand-curated data).
fn existing_file_has_real_data(path: &str) -> bool {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(data) = serde_json::from_str::<ExistingFile>(&contents) else {
        eprintln!(
            "WARNING: Could not parse '{}' as JSON; treating as real data to be safe.",
            path
        );
        return true;
    };
    data.meta.source.as_deref() != Some("template")
}

/// Check if the generated data is template-based (either explicitly requested
/// or produced by a scraping fallback).
fn is_template_data(data: &scraper::DraftOrderData) -> bool {
    data.meta.source == "template"
}

/// Write an RFC 3339 timestamp file alongside the output to track when
/// the draft order was last successfully scraped/generated.
fn write_timestamp_file(output_path: &str) -> Result<()> {
    let parent = Path::new(output_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let timestamp_path = parent.join(".draft_order_last_scraped");
    let now = chrono::Utc::now().to_rfc3339();
    std::fs::write(&timestamp_path, &now)?;
    println!("Wrote timestamp to: {}", timestamp_path.display());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Validate output path early, before doing network I/O
    if let Some(parent) = Path::new(&cli.output).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
            println!("Created output directory: {}", parent.display());
        }
    }

    println!("NFL Draft Order Scraper");
    println!("Year: {}", cli.year);
    println!("Output: {}", cli.output);

    // Safety guard: check early whether existing file has curated data, so we
    // can refuse to overwrite it with template output without waiting for I/O.
    let has_real_data = existing_file_has_real_data(&cli.output);

    let data = if cli.template {
        println!("\nGenerating template draft order...");
        scraper::generate_template_draft_order(cli.year)
    } else {
        println!("\nFetching draft order from Tankathon.com...");
        match scraper::fetch_tankathon_html(cli.year).await {
            Ok(html) => {
                println!("Fetched {} bytes of HTML", html.len());
                match scraper::parse_tankathon_html(&html, cli.year) {
                    Ok(data) if data.draft_order.is_empty() => {
                        println!("\nScraping produced no results. Generating template instead.");
                        println!("Edit the output file manually to match the actual draft order.");
                        scraper::generate_template_draft_order(cli.year)
                    }
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Failed to parse HTML: {}", e);
                        println!("Generating template instead...");
                        scraper::generate_template_draft_order(cli.year)
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch Tankathon: {}", e);
                println!("Generating template instead...");
                scraper::generate_template_draft_order(cli.year)
            }
        }
    };

    // Safety guard: refuse to overwrite curated data with template output
    if is_template_data(&data)
        && !cli.template
        && !cli.allow_template_fallback
        && has_real_data
    {
        eprintln!(
            "\nERROR: Scraping failed and would fall back to template data, but '{}' \
             already contains curated (non-template) data.",
            cli.output
        );
        eprintln!("Refusing to overwrite to protect your curated draft order.");
        eprintln!("To overwrite anyway, pass --allow-template-fallback");
        std::process::exit(1);
    }

    println!("\nDraft order summary:");
    println!("  Year: {}", data.meta.draft_year);
    println!("  Rounds: {}", data.meta.total_rounds);
    println!("  Total picks: {}", data.meta.total_picks);

    let comp_count = data
        .draft_order
        .iter()
        .filter(|e| e.is_compensatory)
        .count();
    println!("  Compensatory picks: {}", comp_count);

    // Write JSON output
    let json = serde_json::to_string_pretty(&data)?;
    std::fs::write(&cli.output, &json)?;
    println!("\nWrote draft order to: {}", cli.output);

    // Write timestamp file for staleness tracking
    write_timestamp_file(&cli.output)?;

    Ok(())
}
