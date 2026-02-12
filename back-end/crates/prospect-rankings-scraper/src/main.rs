mod models;
mod scrapers;
mod template;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

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

    Ok(())
}
