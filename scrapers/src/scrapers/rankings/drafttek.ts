import type { RankingData, RankingEntry } from "../../types/rankings.js";
import { parseDraftTekHtml } from "./drafttek-parser.js";

const BASE_URL = "https://www.drafttek.com/2026-NFL-Draft-Big-Board";
const PAGES = [
  "Top-NFL-Draft-Prospects-2026-Page-1.asp",
  "Top-NFL-Draft-Prospects-2026-Page-2.asp",
  "Top-NFL-Draft-Prospects-2026-Page-3.asp",
  "Top-NFL-Draft-Prospects-2026-Page-4.asp",
  "Top-NFL-Draft-Prospects-2026-Page-5.asp",
];
const POLITE_DELAY_MS = 500;

async function fetchPage(url: string): Promise<string> {
  const response = await fetch(url, {
    headers: {
      "User-Agent":
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    },
    signal: AbortSignal.timeout(30000),
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status} fetching ${url}`);
  }

  return response.text();
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function scrapeDraftTek(year: number): Promise<RankingData> {
  console.error("Scraping DraftTek prospect rankings...");

  const allRankings: RankingEntry[] = [];

  for (let i = 0; i < PAGES.length; i++) {
    const url = `${BASE_URL}/${PAGES[i]}`;
    console.error(`  Fetching page ${i + 1}/${PAGES.length}: ${url}`);

    try {
      const html = await fetchPage(url);
      const pageData = parseDraftTekHtml(html, year);
      console.error(`    Found ${pageData.rankings.length} prospects`);
      allRankings.push(...pageData.rankings);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error(`    Failed: ${message}`);
      break;
    }

    // Polite delay between pages
    if (i < PAGES.length - 1) {
      await sleep(POLITE_DELAY_MS);
    }
  }

  // Re-rank sequentially
  allRankings.forEach((entry, i) => {
    entry.rank = i + 1;
  });

  console.error(`Total: ${allRankings.length} prospects from DraftTek`);

  return {
    meta: {
      version: "1.0.0",
      source: "drafttek",
      source_url: `${BASE_URL}/${PAGES[0]}`,
      draft_year: year,
      scraped_at: new Date().toISOString().slice(0, 10),
      total_prospects: allRankings.length,
    },
    rankings: allRankings,
  };
}
