import { writeJsonFile, shouldPreventOverwrite } from "../shared/json-writer.js";
import { generateTemplateCombine } from "../scrapers/combine/template.js";
import { scrapePfr } from "../scrapers/combine/pfr.js";
import { scrapeMockdraftable } from "../scrapers/combine/mockdraftable.js";
import { scrapeNflverse } from "../scrapers/combine/nflverse.js";
import { scrapeNflCom } from "../scrapers/combine/nfl-com.js";
import { scrapeNflCombineResults } from "../scrapers/combine/nflcombineresults.js";
import { mergeCombineData } from "../scrapers/combine/merge.js";
import { validateCombineData } from "../shared/combine-validator.js";
import type { CombineData } from "../types/combine.js";

export interface CombineOptions {
  year: number;
  output: string;
  template?: boolean;
  source?: string;
  merge?: boolean;
  allowTemplateFallback?: boolean;
  force?: boolean;
}

export async function runCombineCommand(options: CombineOptions): Promise<void> {
  const {
    year,
    output,
    template = false,
    source = "pfr",
    merge = false,
    allowTemplateFallback = false,
    force = false,
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
    data = await scrapeAndMerge(year, allowTemplateFallback);
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

  // Run validation
  if (!template) {
    const validation = validateCombineData(data);
    for (const warning of validation.warnings) {
      console.error(`WARNING: ${warning}`);
    }
    for (const error of validation.errors) {
      console.error(`VALIDATION ERROR: ${error}`);
    }
    if (validation.errors.length > 0 && !force) {
      throw new Error(
        `Data quality validation failed with ${validation.errors.length} error(s). ` +
          "Pass --force to write anyway.",
      );
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
  switch (source) {
    case "pfr":
      return await scrapePfr(year);
    case "mockdraftable":
      return await scrapeMockdraftable(year);
    case "nflverse":
      return await scrapeNflverse(year);
    case "nfl-com":
      return await scrapeNflCom(year);
    case "nflcombineresults":
      return await scrapeNflCombineResults(year);
    default:
      throw new Error(
        `Unknown source: ${source}. Use pfr, mockdraftable, nflverse, nfl-com, or nflcombineresults`,
      );
  }
}

async function scrapeAndMerge(
  year: number,
  allowTemplateFallback: boolean,
): Promise<CombineData> {
  let primary: CombineData | null = null;
  const secondaries: CombineData[] = [];

  // Primary: nflverse (most reliable, structured CSV)
  try {
    console.error("\n[1/5] Scraping nflverse (primary)...");
    primary = await scrapeNflverse(year);
    console.error(`  Got ${primary.combine_results.length} players from nflverse`);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`nflverse failed: ${message}`);
  }

  // Secondary: NFL.com (official source, Playwright-based)
  try {
    console.error("\n[2/5] Scraping NFL.com Combine Tracker...");
    const nflCom = await scrapeNflCom(year);
    if (nflCom.combine_results.length > 0) {
      if (primary) {
        secondaries.push(nflCom);
        console.error(`  Got ${nflCom.combine_results.length} players from NFL.com (secondary)`);
      } else {
        primary = nflCom;
        console.error(
          `  Got ${nflCom.combine_results.length} players from NFL.com (promoted to primary)`,
        );
      }
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`NFL.com failed: ${message}`);
  }

  // Tertiary: PFR (backfills arm_length, hand_size, wingspan, splits)
  try {
    console.error("\n[3/5] Scraping Pro Football Reference...");
    const pfr = await scrapePfr(year);
    if (pfr.combine_results.length > 0) {
      if (primary) {
        secondaries.push(pfr);
        console.error(`  Got ${pfr.combine_results.length} players from PFR (secondary)`);
      } else {
        primary = pfr;
        console.error(`  Got ${pfr.combine_results.length} players from PFR (promoted to primary)`);
      }
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`PFR failed: ${message}`);
  }

  // Quaternary: nflcombineresults.com
  try {
    console.error("\n[4/5] Scraping nflcombineresults.com...");
    const ncr = await scrapeNflCombineResults(year);
    if (ncr.combine_results.length > 0) {
      if (primary) {
        secondaries.push(ncr);
        console.error(
          `  Got ${ncr.combine_results.length} players from nflcombineresults.com (secondary)`,
        );
      } else {
        primary = ncr;
        console.error(
          `  Got ${ncr.combine_results.length} players from nflcombineresults.com (promoted to primary)`,
        );
      }
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`nflcombineresults.com failed: ${message}`);
  }

  // Quinary: Mockdraftable
  try {
    console.error("\n[5/5] Scraping Mockdraftable...");
    const md = await scrapeMockdraftable(year);
    if (md.combine_results.length > 0) {
      if (primary) {
        secondaries.push(md);
        console.error(
          `  Got ${md.combine_results.length} players from Mockdraftable (secondary)`,
        );
      } else {
        primary = md;
        console.error(
          `  Got ${md.combine_results.length} players from Mockdraftable (promoted to primary)`,
        );
      }
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error(`Mockdraftable failed: ${message}`);
  }

  // If all sources failed, only use template if explicitly allowed
  if (!primary) {
    if (allowTemplateFallback) {
      console.error("\nAll sources failed. Using template as fallback...");
      primary = generateTemplateCombine(year);
    } else {
      throw new Error(
        "All combine data sources failed. Pass --allow-template-fallback to use template data.",
      );
    }
  }

  return mergeCombineData(primary, secondaries);
}
