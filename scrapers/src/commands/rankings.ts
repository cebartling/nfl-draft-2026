import { writeJsonFile, shouldPreventOverwrite } from "../shared/json-writer.js";
import { generateTemplateRankings } from "../scrapers/rankings/template.js";
import { scrapeTankathonRankings } from "../scrapers/rankings/tankathon.js";
import { scrapeDraftTek } from "../scrapers/rankings/drafttek.js";
import { scrapeWalterFootball } from "../scrapers/rankings/walterfootball.js";
import { mergeRankings } from "../scrapers/rankings/merge.js";
import type { RankingData } from "../types/rankings.js";

export interface RankingsOptions {
  year: number;
  output: string;
  template?: boolean;
  source?: string;
  merge?: boolean;
  allowTemplateFallback?: boolean;
}

export async function runRankingsCommand(options: RankingsOptions): Promise<void> {
  const {
    year,
    output,
    template = false,
    source = "tankathon",
    merge = false,
    allowTemplateFallback = false,
  } = options;

  console.error("NFL Prospect Rankings Scraper");
  console.error(`Year: ${year}`);
  console.error(`Output: ${output}`);

  let data: RankingData;

  if (template) {
    console.error("\nGenerating template rankings...");
    data = generateTemplateRankings(year);
  } else if (merge) {
    console.error("\nMerging rankings from multiple sources...");
    data = await scrapeAndMerge(year);
  } else {
    console.error(`\nScraping from: ${source}`);
    data = await scrapeSource(source, year);
  }

  // Safety guard: refuse to overwrite curated data with template output
  if (data.meta.source === "template" && !template && !allowTemplateFallback) {
    if (shouldPreventOverwrite(output, { meta: { source: data.meta.source } })) {
      console.error(
        `\nERROR: Scraping failed and would fall back to template data, but '${output}' ` +
          "already contains curated (non-template) data.",
      );
      console.error("Refusing to overwrite to protect your curated rankings.");
      console.error("To overwrite anyway, pass --allow-template-fallback");
      throw new Error("Refusing to overwrite curated data with template");
    }
  }

  console.error("\nRankings summary:");
  console.error(`  Year: ${data.meta.draft_year}`);
  console.error(`  Source: ${data.meta.source}`);
  console.error(`  Total prospects: ${data.meta.total_prospects}`);

  writeJsonFile(output, data);
  console.error(`\nWrote rankings to: ${output}`);
}

async function scrapeSource(source: string, year: number): Promise<RankingData> {
  try {
    switch (source) {
      case "tankathon":
        return await scrapeTankathonRankings(year);
      case "drafttek":
        return await scrapeDraftTek(year);
      case "walterfootball":
        return await scrapeWalterFootball(year);
      default:
        throw new Error(`Unknown source: ${source}. Use tankathon, drafttek, or walterfootball`);
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`Failed to scrape ${source}: ${message}`);
    console.error("Generating template instead...");
    return generateTemplateRankings(year);
  }
}

async function scrapeAndMerge(year: number): Promise<RankingData> {
  let primary: RankingData;
  const secondaries: RankingData[] = [];

  // Primary: Tankathon
  try {
    console.error("\n[1/3] Scraping Tankathon (primary)...");
    primary = await scrapeTankathonRankings(year);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`Tankathon failed: ${message}`);
    console.error("Using template as primary...");
    primary = generateTemplateRankings(year);
  }

  // Secondary: DraftTek
  try {
    console.error("\n[2/3] Scraping DraftTek...");
    const drafttek = await scrapeDraftTek(year);
    if (drafttek.rankings.length > 0) {
      secondaries.push(drafttek);
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`DraftTek failed: ${message}`);
  }

  // Secondary: WalterFootball
  try {
    console.error("\n[3/3] Scraping WalterFootball...");
    const wf = await scrapeWalterFootball(year);
    if (wf.rankings.length > 0) {
      secondaries.push(wf);
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`WalterFootball failed: ${message}`);
  }

  return mergeRankings(primary, secondaries);
}
