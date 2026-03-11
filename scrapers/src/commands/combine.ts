import { writeJsonFile, shouldPreventOverwrite } from "../shared/json-writer.js";
import { generateTemplateCombine } from "../scrapers/combine/template.js";
import { scrapePfr } from "../scrapers/combine/pfr.js";
import { scrapeMockdraftable } from "../scrapers/combine/mockdraftable.js";
import { mergeCombineData } from "../scrapers/combine/merge.js";
import type { CombineData } from "../types/combine.js";

export interface CombineOptions {
  year: number;
  output: string;
  template?: boolean;
  source?: string;
  merge?: boolean;
  allowTemplateFallback?: boolean;
}

export async function runCombineCommand(options: CombineOptions): Promise<void> {
  const {
    year,
    output,
    template = false,
    source = "pfr",
    merge = false,
    allowTemplateFallback = false,
  } = options;

  console.error("NFL Combine Data Scraper");
  console.error(`Year: ${year}`);
  console.error(`Output: ${output}`);

  let data: CombineData;

  if (template) {
    console.error("\nGenerating template combine data...");
    data = generateTemplateCombine(year);
  } else if (merge) {
    console.error("\nMerging combine data from multiple sources...");
    data = await scrapeAndMerge(year);
  } else {
    console.error(`\nScraping from: ${source}`);
    data = await scrapeSource(source, year);
  }

  // Safety guard
  if (data.meta.source === "template" && !template && !allowTemplateFallback) {
    if (shouldPreventOverwrite(output, { meta: { source: data.meta.source } })) {
      console.error(
        `\nERROR: Scraping failed and would fall back to template data, but '${output}' ` +
          "already contains curated (non-template) data.",
      );
      console.error("Refusing to overwrite to protect your curated combine data.");
      console.error("To overwrite anyway, pass --allow-template-fallback");
      throw new Error("Refusing to overwrite curated data with template");
    }
  }

  console.error("\nCombine data summary:");
  console.error(`  Year: ${data.meta.year}`);
  console.error(`  Source: ${data.meta.source}`);
  console.error(`  Players: ${data.meta.player_count}`);
  console.error(`  Entries: ${data.meta.entry_count}`);

  writeJsonFile(output, data);
  console.error(`\nWrote combine data to: ${output}`);
}

async function scrapeSource(source: string, year: number): Promise<CombineData> {
  try {
    switch (source) {
      case "pfr":
        return await scrapePfr(year);
      case "mockdraftable":
        return await scrapeMockdraftable(year);
      default:
        throw new Error(`Unknown source: ${source}. Use pfr or mockdraftable`);
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`Failed to scrape ${source}: ${message}`);
    console.error("Generating template instead...");
    return generateTemplateCombine(year);
  }
}

async function scrapeAndMerge(year: number): Promise<CombineData> {
  let primary: CombineData;
  const secondaries: CombineData[] = [];

  // Primary: PFR
  try {
    console.error("\n[1/2] Scraping Pro Football Reference (primary)...");
    primary = await scrapePfr(year);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`PFR failed: ${message}`);
    console.error("Using template as primary...");
    primary = generateTemplateCombine(year);
  }

  // Secondary: Mockdraftable
  try {
    console.error("\n[2/2] Scraping Mockdraftable...");
    const md = await scrapeMockdraftable(year);
    if (md.combine_results.length > 0) {
      secondaries.push(md);
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`Mockdraftable failed: ${message}`);
  }

  return mergeCombineData(primary, secondaries);
}
