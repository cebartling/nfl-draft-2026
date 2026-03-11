import type { CombineData } from "../../types/combine.js";
import { extractInitialState, parseInitialState } from "./mockdraftable-parser.js";

export function combineUrl(year: number): string {
  return `https://www.mockdraftable.com/search?year=${year}&beginYear=${year}&endYear=${year}&sort=name`;
}

export async function scrapeMockdraftable(year: number): Promise<CombineData> {
  const url = combineUrl(year);
  console.error(`Fetching Mockdraftable combine data from: ${url}`);

  const response = await fetch(url, {
    headers: {
      "User-Agent":
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    },
    signal: AbortSignal.timeout(30000),
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status} fetching Mockdraftable combine data`);
  }

  const html = await response.text();
  console.error(`Fetched ${html.length} bytes of HTML`);

  const json = extractInitialState(html);
  return parseInitialState(json, year);
}
