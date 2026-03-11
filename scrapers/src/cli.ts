#!/usr/bin/env bun

const args = process.argv.slice(2);
const command = args[0];

function getArg(name: string, defaultValue: string): string {
  const idx = args.indexOf(name);
  return idx !== -1 && idx + 1 < args.length ? args[idx + 1] : defaultValue;
}

function hasFlag(name: string): boolean {
  return args.includes(name);
}

async function main() {
  switch (command) {
    case "draft-order": {
      const { runDraftOrderCommand } = await import("./commands/draft-order.js");
      await runDraftOrderCommand({
        year: parseInt(getArg("--year", "2026"), 10),
        output: getArg("--output", "../back-end/data/draft_order_2026.json"),
        template: hasFlag("--template"),
        allowTemplateFallback: hasFlag("--allow-template-fallback"),
      });
      break;
    }

    default:
      console.error("Usage: bun run scrape <command> [options]");
      console.error("");
      console.error("Commands:");
      console.error("  draft-order    Scrape NFL draft order from Tankathon");
      console.error("  combine        Scrape NFL Combine data (coming soon)");
      console.error("  rankings       Scrape prospect rankings (coming soon)");
      console.error("");
      console.error("Options:");
      console.error("  --year <year>       Draft year (default: 2026)");
      console.error("  --output <path>     Output file path");
      console.error("  --template          Generate template without scraping");
      process.exit(1);
  }
}

main().catch((err) => {
  console.error(`ERROR: ${err.message}`);
  process.exit(1);
});
