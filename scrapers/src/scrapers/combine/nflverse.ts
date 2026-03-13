import type { CombineData } from "../../types/combine.js";
import { parseNflverseCsv } from "./nflverse-parser.js";

export const combineUrl =
  "https://github.com/nflverse/nflverse-data/releases/download/combine/combine.csv";

export async function scrapeNflverse(year: number): Promise<CombineData> {
  console.error(`Fetching nflverse combine data from: ${combineUrl}`);

  const response = await fetch(combineUrl, {
    headers: {
      "User-Agent":
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    },
    signal: AbortSignal.timeout(60000),
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status} fetching nflverse combine data from ${combineUrl}`);
  }

  const csvText = await response.text();
  console.error(`Fetched ${csvText.length} bytes of CSV`);

  return parseNflverseCsv(csvText, year);
}
