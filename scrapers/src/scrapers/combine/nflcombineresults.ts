import type { CombineData } from "../../types/combine.js";
import { parseNflCombineResultsHtml } from "./nflcombineresults-parser.js";

export function combineUrl(year: number): string {
  return `https://nflcombineresults.com/nflcombinedata.php?year=${year}`;
}

/**
 * Scrape nflcombineresults.com combine data (cheerio-based HTML table scraping).
 */
export async function scrapeNflCombineResults(year: number): Promise<CombineData> {
  const url = combineUrl(year);
  console.error(`Fetching nflcombineresults.com combine data from: ${url}`);

  const response = await fetch(url, {
    headers: {
      "User-Agent":
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    },
    signal: AbortSignal.timeout(30000),
  });

  if (response.status === 404) {
    throw new Error(
      "nflcombineresults.com returned 404 — site may be down or URL structure changed.",
    );
  }

  if (!response.ok) {
    throw new Error(`HTTP ${response.status} fetching nflcombineresults.com combine data`);
  }

  const html = await response.text();
  console.error(`Fetched ${html.length} bytes of HTML`);

  const data = parseNflCombineResultsHtml(html, year);
  console.error(`Extracted ${data.combine_results.length} combine entries from nflcombineresults.com`);

  if (data.combine_results.length === 0 && html.length > 1000) {
    const hasTable = html.includes("datatable");
    throw new Error(
      `nflcombineresults.com returned ${html.length} bytes of HTML but parser extracted 0 entries. ` +
        `Data table ${hasTable ? "was" : "was NOT"} found in HTML. ` +
        "The page structure may have changed.",
    );
  }

  return data;
}
