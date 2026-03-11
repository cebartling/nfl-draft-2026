import type { CombineData } from "../../types/combine.js";
import { parsePfrHtml } from "./pfr-parser.js";

export function combineUrl(year: number): string {
  return `https://www.pro-football-reference.com/draft/${year}-combine.htm`;
}

export async function scrapePfr(year: number): Promise<CombineData> {
  const url = combineUrl(year);
  console.error(`Fetching PFR combine data from: ${url}`);

  const response = await fetch(url, {
    headers: {
      "User-Agent":
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    },
    signal: AbortSignal.timeout(30000),
  });

  if (response.status === 403) {
    throw new Error("PFR returned 403 Forbidden. Try using --browser flag for Playwright-based scraping.");
  }

  if (!response.ok) {
    throw new Error(`HTTP ${response.status} fetching PFR combine data`);
  }

  const html = await response.text();
  console.error(`Fetched ${html.length} bytes of HTML`);

  return parsePfrHtml(html, year);
}
