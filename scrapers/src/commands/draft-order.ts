import { writeJsonFile, shouldPreventOverwrite } from "../shared/json-writer.js";
import { generateTemplateDraftOrder } from "../scrapers/draft-order/template.js";
import { scrapeTankathon } from "../scrapers/draft-order/tankathon.js";
import type { DraftOrderData } from "../types/draft-order.js";

export interface DraftOrderOptions {
  year: number;
  output: string;
  template?: boolean;
  allowTemplateFallback?: boolean;
}

export async function runDraftOrderCommand(options: DraftOrderOptions): Promise<void> {
  const { year, output, template = false, allowTemplateFallback = false } = options;

  console.error("NFL Draft Order Scraper");
  console.error(`Year: ${year}`);
  console.error(`Output: ${output}`);

  let data: DraftOrderData;

  if (template) {
    console.error("\nGenerating template draft order...");
    data = generateTemplateDraftOrder(year);
  } else {
    console.error("\nFetching draft order from Tankathon.com...");
    try {
      data = await scrapeTankathon(year);

      if (data.draft_order.length === 0) {
        console.error("\nScraping produced no results. Generating template instead.");
        data = generateTemplateDraftOrder(year);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error(`Failed to scrape Tankathon: ${message}`);
      console.error("Generating template instead...");
      data = generateTemplateDraftOrder(year);
    }
  }

  // Safety guard: refuse to overwrite curated data with template output
  if (data.meta.source === "template" && !template && !allowTemplateFallback) {
    if (shouldPreventOverwrite(output, { meta: { source: data.meta.source } })) {
      console.error(
        `\nERROR: Scraping failed and would fall back to template data, but '${output}' ` +
          "already contains curated (non-template) data.",
      );
      console.error("Refusing to overwrite to protect your curated draft order.");
      console.error("To overwrite anyway, pass --allow-template-fallback");
      throw new Error("Refusing to overwrite curated data with template");
    }
  }

  const compCount = data.draft_order.filter((e) => e.is_compensatory).length;
  console.error("\nDraft order summary:");
  console.error(`  Year: ${data.meta.draft_year}`);
  console.error(`  Rounds: ${data.meta.total_rounds}`);
  console.error(`  Total picks: ${data.meta.total_picks}`);
  console.error(`  Compensatory picks: ${compCount}`);

  writeJsonFile(output, data);
  console.error(`\nWrote draft order to: ${output}`);
}
