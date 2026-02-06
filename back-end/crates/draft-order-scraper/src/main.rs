mod scraper;
#[allow(dead_code)]
mod team_name_mapper;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "draft-order-scraper")]
#[command(about = "Scrape NFL draft order from Tankathon.com and generate JSON")]
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("NFL Draft Order Scraper");
    println!("Year: {}", cli.year);
    println!("Output: {}", cli.output);

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

    Ok(())
}
