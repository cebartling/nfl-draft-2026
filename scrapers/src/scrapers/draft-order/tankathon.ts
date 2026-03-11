import type { DraftOrderData } from "../../types/draft-order.js";
import { parseTankathonHtml } from "./parser.js";

export async function scrapeTankathon(year: number): Promise<DraftOrderData> {
  const { chromium } = await import("playwright");

  const url =
    year === 2026
      ? "https://www.tankathon.com/nfl/full_draft"
      : `https://www.tankathon.com/nfl/full_draft/${year}`;

  console.error(`Scraping Tankathon full draft order...`);
  console.error(`URL: ${url}`);

  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    await page.goto(url, { waitUntil: "domcontentloaded", timeout: 60000 });
    await page.waitForSelector("div.full-draft-round table.full-draft", { timeout: 30000 });

    const html = await page.content();
    console.error(`Fetched ${html.length} bytes of HTML`);

    const data = parseTankathonHtml(html, year);

    if (data.draft_order.length === 0) {
      console.error("WARNING: No picks extracted from Tankathon page");
    } else {
      console.error(`Extracted ${data.draft_order.length} picks`);
      if (data.draft_order.length < 200) {
        console.error(
          `WARNING: Only ${data.draft_order.length} picks (expected 220+). Some rounds may not have been extracted.`,
        );
      }
    }

    return data;
  } finally {
    await browser.close();
  }
}
