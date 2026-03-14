import type { CombineData } from "../../types/combine.js";
import { parseNflComHtml } from "./nfl-com-parser.js";
import { fetchRenderedPage, closeBrowser } from "../../shared/browser.js";

export function combineUrl(_year: number): string {
  return "https://www.nfl.com/combine/tracker/live-results/";
}

/**
 * Scrape NFL.com combine tracker using Playwright (client-side rendered page).
 */
export async function scrapeNflCom(year: number): Promise<CombineData> {
  const url = combineUrl(year);
  console.error(`Fetching NFL.com combine data from: ${url}`);
  console.error("Using Playwright to render client-side content...");

  try {
    const html = await fetchRenderedPage(url, "table", 30000);
    console.error(`Rendered page: ${html.length} bytes of HTML`);

    const data = parseNflComHtml(html, year);
    console.error(`Extracted ${data.combine_results.length} combine entries from NFL.com`);

    if (data.combine_results.length === 0 && html.length > 1000) {
      throw new Error(
        `NFL.com returned ${html.length} bytes of HTML but parser extracted 0 entries. ` +
          "The page structure may have changed.",
      );
    }

    return data;
  } finally {
    await closeBrowser();
  }
}
