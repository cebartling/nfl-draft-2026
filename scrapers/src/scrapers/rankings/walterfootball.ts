import type { RankingData } from "../../types/rankings.js";
import { parseWalterFootballHtml } from "./walterfootball-parser.js";

export async function scrapeWalterFootball(year: number): Promise<RankingData> {
  const url = `https://walterfootball.com/nfldraftbigboard${year}.php`;

  console.error(`Scraping WalterFootball big board...`);
  console.error(`URL: ${url}`);

  const response = await fetch(url, {
    headers: {
      "User-Agent":
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    },
    signal: AbortSignal.timeout(30000),
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status} fetching WalterFootball`);
  }

  const html = await response.text();
  console.error(`Fetched ${html.length} bytes of HTML`);

  const data = parseWalterFootballHtml(html, year);

  if (data.rankings.length === 0) {
    console.error("WARNING: No prospects extracted from WalterFootball");
  } else {
    console.error(`Extracted ${data.rankings.length} prospects`);
  }

  return data;
}
