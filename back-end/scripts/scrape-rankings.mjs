#!/usr/bin/env node
/**
 * Playwright-based scraper for Tankathon NFL Big Board.
 *
 * Tankathon is a JS-rendered SPA, so reqwest/scraper can't parse the DOM.
 * This script uses Playwright to render the page and extract prospect data.
 *
 * Usage:
 *   node scrape-rankings.mjs [--output path] [--year 2026]
 *
 * Output: JSON file in RankingData format compatible with seed-data scouting loader.
 */

import { writeFileSync } from "fs";
import { resolve, dirname, join } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// Resolve playwright from multiple locations (project front-end, global, local)
let chromium;
for (const candidate of [
  join(__dirname, "../../front-end/node_modules/playwright/index.mjs"),
  "playwright",
]) {
  try {
    ({ chromium } = await import(candidate));
    break;
  } catch {
    // try next
  }
}
if (!chromium) {
  console.error(
    "Could not find playwright. Install it: npm install playwright",
  );
  process.exit(1);
}

const args = process.argv.slice(2);
function getArg(name, defaultValue) {
  const idx = args.indexOf(name);
  return idx !== -1 && idx + 1 < args.length ? args[idx + 1] : defaultValue;
}

const year = parseInt(getArg("--year", "2026"), 10);
const outputPath = resolve(
  getArg("--output", `data/rankings/rankings_${year}.json`),
);

const url =
  year === 2026
    ? "https://www.tankathon.com/nfl/big_board"
    : `https://www.tankathon.com/nfl/big_board/${year}`;

console.error(`Scraping Tankathon big board for ${year}...`);
console.error(`URL: ${url}`);

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();

try {
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 60000 });

  // Wait for prospect rows to render inside the main big board section
  await page.waitForSelector("#big-board div.mock-row.nfl", { timeout: 30000 });

  // Extract prospect data only from #big-board (excludes by-school/conference duplicates)
  const allProspects = await page.$$eval(
    "#big-board div.mock-row.nfl",
    (rows) => {
      return rows.map((row, index) => {
        const rankEl = row.querySelector("div.mock-row-pick-number");
        const nameEl = row.querySelector("div.mock-row-name");
        const schoolPosEl = row.querySelector("div.mock-row-school-position");

        const rankText = rankEl?.textContent?.trim() ?? "";
        const rank = parseInt(rankText, 10) || index + 1;

        const fullName = nameEl?.textContent?.trim() ?? "";
        const nameParts = fullName.split(/\s+/);
        const firstName = nameParts[0] ?? "";
        const lastName = nameParts.slice(1).join(" ");

        // Format: "Position | School"  or  "Position | School | #Jersey"
        const schoolPosText = schoolPosEl?.textContent?.trim() ?? "";
        const spParts = schoolPosText.split("|").map((s) => s.trim());
        const position = spParts[0] ?? "";
        const school = spParts[1] ?? "";

        return { rank, firstName, lastName, position, school };
      });
    },
  );

  // Filter out future draft class entries (rank looks like a year: 2027, 2028, etc.)
  // and normalize positions that the seed-data position mapper doesn't accept
  const positionMap = {
    DL: "DT",
    IOL: "OG",
    "EDGE/LB": "EDGE",
    "LB/EDGE": "EDGE",
    OL: "OT",
    DB: "CB",
    ATH: "WR",
    FB: "RB",
  };
  const prospects = allProspects
    .filter((p) => p.rank < 2025)
    .map((p) => ({
      ...p,
      position: positionMap[p.position] ?? p.position,
    }));

  console.error(
    `Extracted ${allProspects.length} rows, ${prospects.length} current-class prospects (filtered ${allProspects.length - prospects.length} future-class entries)`,
  );

  // Build RankingData JSON
  const today = new Date().toISOString().slice(0, 10);
  const data = {
    meta: {
      version: "1.0.0",
      source: "tankathon",
      source_url: url,
      draft_year: year,
      scraped_at: today,
      total_prospects: prospects.length,
    },
    rankings: prospects.map((p) => ({
      rank: p.rank,
      first_name: p.firstName,
      last_name: p.lastName,
      position: p.position,
      school: p.school,
    })),
  };

  writeFileSync(outputPath, JSON.stringify(data, null, 2) + "\n");
  console.error(`Wrote ${prospects.length} prospects to ${outputPath}`);

  // Print top 10 for quick verification
  console.error("\nTop 10:");
  for (const p of prospects.slice(0, 10)) {
    console.error(
      `  ${p.rank}. ${p.firstName} ${p.lastName} - ${p.position} (${p.school})`,
    );
  }
} catch (err) {
  console.error(`Scraping failed: ${err.message}`);
  process.exit(1);
} finally {
  await browser.close();
}
