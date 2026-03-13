#!/usr/bin/env bun

const args = process.argv.slice(2);
const command = args[0];

function getArg(name: string, defaultValue: string): string {
  const idx = args.indexOf(name);
  if (idx === -1 || idx + 1 >= args.length) return defaultValue;
  const value = args[idx + 1];
  return value.startsWith("--") ? defaultValue : value;
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

    case "rankings": {
      const { runRankingsCommand } = await import("./commands/rankings.js");
      await runRankingsCommand({
        year: parseInt(getArg("--year", "2026"), 10),
        output: getArg("--output", "../back-end/data/prospect_rankings_2026.json"),
        template: hasFlag("--template"),
        source: getArg("--source", "tankathon"),
        merge: hasFlag("--merge"),
        allowTemplateFallback: hasFlag("--allow-template-fallback"),
      });
      break;
    }

    case "combine": {
      const { runCombineCommand } = await import("./commands/combine.js");
      await runCombineCommand({
        year: parseInt(getArg("--year", "2026"), 10),
        output: getArg("--output", "../back-end/data/combine_2026.json"),
        template: hasFlag("--template"),
        source: getArg("--source", "pfr"),
        merge: hasFlag("--merge"),
        allowTemplateFallback: hasFlag("--allow-template-fallback"),
        force: hasFlag("--force"),
      });
      break;
    }

    default:
      console.error("Usage: bun run scrape <command> [options]");
      console.error("");
      console.error("Commands:");
      console.error("  draft-order    Scrape NFL draft order from Tankathon");
      console.error("  rankings       Scrape prospect rankings");
      console.error("  combine        Scrape NFL Combine data");
      console.error("");
      console.error("Options:");
      console.error("  --year <year>       Draft year (default: 2026)");
      console.error("  --output <path>     Output file path");
      console.error("  --template          Generate template without scraping");
      console.error("  --source <name>     Source (rankings: tankathon|drafttek|walterfootball; combine: pfr|mockdraftable|nflverse)");
      console.error("  --merge             Merge data from all sources");
      console.error("  --force             Write output even if validation fails");
      process.exit(1);
  }
}

main().catch((err) => {
  console.error(`ERROR: ${err.message}`);
  process.exit(1);
});
