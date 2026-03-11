import type { RankingData } from "../../types/rankings.js";
import { parseTankathonRankingsHtml } from "./tankathon-parser.js";

export async function scrapeTankathonRankings(year: number): Promise<RankingData> {
  const { chromium } = await import("playwright");

  const url = "https://www.tankathon.com/nfl/big_board";

  console.error(`Scraping Tankathon big board...`);
  console.error(`URL: ${url}`);

  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    await page.goto(url, { waitUntil: "domcontentloaded", timeout: 60000 });

    // Wait for mock-row elements to appear
    await page.waitForSelector("div.mock-row.nfl", { timeout: 30000 }).catch(() => {
      console.error("WARNING: mock-row elements not found, will try embedded JSON fallback");
    });

    const html = await page.content();
    console.error(`Fetched ${html.length} bytes of HTML`);

    const data = parseTankathonRankingsHtml(html, year);

    if (data.rankings.length === 0) {
      console.error("WARNING: No prospects extracted from Tankathon big board");
    } else {
      console.error(`Extracted ${data.rankings.length} prospects`);
    }

    return data;
  } finally {
    await browser.close();
  }
}
