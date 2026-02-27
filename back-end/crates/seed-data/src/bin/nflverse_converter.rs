use anyhow::Result;
use clap::Parser;
use seed_data::nflverse_converter::{convert_csv_file, write_combine_json};

/// Convert nflverse combine CSV data to the project's combine JSON format.
///
/// Downloads the CSV from:
/// https://github.com/nflverse/nflverse-data/releases/download/combine/combine.csv
///
/// Then run this tool to convert it:
///   cargo run -p seed-data --bin nflverse_converter -- --input data/nflverse_combine.csv
#[derive(Parser, Debug)]
#[command(name = "nflverse-converter", about = "Convert nflverse combine CSV to project JSON format")]
struct Args {
    /// Path to the downloaded nflverse combine CSV file
    #[arg(short, long)]
    input: String,

    /// Output JSON file path
    #[arg(short, long, default_value = "data/combine_2026_nflverse.json")]
    output: String,

    /// Season year to filter from the CSV
    #[arg(short, long, default_value_t = 2026)]
    year: i32,

    /// Source field value (combine or pro_day)
    #[arg(short, long, default_value = "combine")]
    source: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("nflverse Combine CSV Converter");
    println!("  Input:  {}", args.input);
    println!("  Output: {}", args.output);
    println!("  Year:   {}", args.year);
    println!("  Source:  {}", args.source);

    let (data, stats) = convert_csv_file(&args.input, args.year, &args.source)?;
    stats.print_summary();

    if stats.converted == 0 {
        println!("\nNo data to write. Check the year filter or CSV contents.");
        return Ok(());
    }

    write_combine_json(&data, &args.output)?;
    println!("\nWrote {} entries to {}", stats.converted, args.output);

    Ok(())
}
