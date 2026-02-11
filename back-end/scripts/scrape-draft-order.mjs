#!/usr/bin/env node
/**
 * Playwright-based scraper for Tankathon NFL Full Draft Order.
 *
 * Tankathon is a JS-rendered SPA, so static HTTP scraping cannot work.
 * This script uses Playwright to render the page and extract draft pick data
 * including traded picks and compensatory picks.
 *
 * Usage:
 *   node scrape-draft-order.js [--output path]
 *
 * Output: JSON file in DraftOrderData format compatible with seed-data loader.
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

const outputPath = resolve(
  getArg("--output", join(__dirname, "../data/draft_order_2026.json")),
);

const url = "https://www.tankathon.com/nfl/full_draft";

// Map Tankathon abbreviations to our project abbreviations
const ABBR_MAP = {
  wsh: "WAS",
};

function normalizeAbbr(abbr) {
  const upper = abbr.toUpperCase();
  return ABBR_MAP[abbr.toLowerCase()] || upper;
}

console.error(`Scraping Tankathon full draft order...`);
console.error(`URL: ${url}`);

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();

try {
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 60000 });

  // Wait for draft round tables to render
  await page.waitForSelector("div.full-draft-round table.full-draft", {
    timeout: 30000,
  });

  // Extract all draft picks from every round
  const rawPicks = await page.$$eval(
    "div.full-draft-round.full-draft-round-nfl",
    (rounds) => {
      const picks = [];

      rounds.forEach((roundDiv, roundIndex) => {
        const roundNum = roundIndex + 1;
        const rows = roundDiv.querySelectorAll("table.full-draft tbody tr");

        rows.forEach((row) => {
          const cells = row.querySelectorAll("td");
          if (cells.length < 2) return;

          const pickCell = cells[0];
          const teamCell = cells[1];

          // Extract overall pick number (text content minus any span text)
          const pickText = pickCell.childNodes[0]?.textContent?.trim() || "";
          const overallPick = parseInt(pickText, 10);
          if (isNaN(overallPick)) return;

          // Check for compensatory pick indicator
          const isCompensatory = !!pickCell.querySelector(
            '[data-balloon="Compensatory pick"]',
          );

          // Extract team abbreviation from div.team-link img src
          // Image src format: http://d2uki2uvp6v3wr.cloudfront.net/nfl/{abbr}.svg
          const teamImg = teamCell.querySelector(
            "div.team-link img.logo-thumb",
          );
          const teamImgSrc = teamImg?.getAttribute("src") || "";
          const teamAbbrMatch = teamImgSrc.match(/\/nfl\/([a-z0-9]+)\.svg/i);
          if (!teamAbbrMatch) {
            throw new Error(
              `Could not extract team abbreviation for pick ${overallPick} (img src: "${teamImgSrc}")`,
            );
          }
          const teamAbbr = teamAbbrMatch[1];

          // Check for traded pick: div.trade contains a link with the original team's img
          let originalTeamAbbr = teamAbbr;
          const tradeDiv = teamCell.querySelector("div.trade");
          if (tradeDiv) {
            const tradeImg = tradeDiv.querySelector("img.logo-thumb");
            const tradeImgSrc = tradeImg?.getAttribute("src") || "";
            const tradeMatch = tradeImgSrc.match(/\/nfl\/([a-z0-9]+)\.svg/i);
            if (tradeMatch) {
              originalTeamAbbr = tradeMatch[1];
            }
          }

          picks.push({
            round: roundNum,
            overallPick,
            teamAbbr,
            originalTeamAbbr,
            isCompensatory,
          });
        });
      });

      return picks;
    },
  );

  const roundCount = new Set(rawPicks.map((p) => p.round)).size;
  console.error(`Extracted ${rawPicks.length} raw picks from ${roundCount} rounds`);

  if (roundCount !== 7) {
    throw new Error(
      `Expected 7 rounds but found ${roundCount}. Page structure may have changed.`,
    );
  }

  // Normalize abbreviations and compute pick_in_round
  const roundCounters = {};
  const draftOrder = rawPicks.map((pick) => {
    const round = pick.round;
    if (!roundCounters[round]) roundCounters[round] = 0;
    roundCounters[round]++;

    const teamAbbr = normalizeAbbr(pick.teamAbbr);
    const origAbbr = normalizeAbbr(pick.originalTeamAbbr);
    const isTrade = teamAbbr !== origAbbr;

    // Build notes
    const noteParts = [];
    if (isTrade) noteParts.push(`From ${origAbbr}`);
    if (pick.isCompensatory) noteParts.push("Compensatory pick");
    const notes = noteParts.length > 0 ? noteParts.join("; ") : null;

    return {
      round,
      pick_in_round: roundCounters[round],
      overall_pick: pick.overallPick,
      team_abbreviation: teamAbbr,
      original_team_abbreviation: origAbbr,
      is_compensatory: pick.isCompensatory,
      notes,
    };
  });

  // Build output JSON
  const today = new Date().toISOString().slice(0, 10);
  const data = {
    meta: {
      version: "1.0.0",
      last_updated: today,
      sources: ["Tankathon.com"],
      draft_year: 2026,
      total_rounds: 7,
      total_picks: draftOrder.length,
    },
    draft_order: draftOrder,
  };

  writeFileSync(outputPath, JSON.stringify(data, null, 2) + "\n");
  console.error(`\nWrote ${draftOrder.length} picks to ${outputPath}`);

  // Print summary stats
  const tradedCount = draftOrder.filter(
    (p) => p.team_abbreviation !== p.original_team_abbreviation,
  ).length;
  const compCount = draftOrder.filter((p) => p.is_compensatory).length;
  console.error(`  Traded picks: ${tradedCount}`);
  console.error(`  Compensatory picks: ${compCount}`);

  // Print picks per round
  for (let r = 1; r <= 7; r++) {
    const count = draftOrder.filter((p) => p.round === r).length;
    console.error(`  Round ${r}: ${count} picks`);
  }

  // Print first 10 picks for verification
  console.error("\nFirst 10 picks:");
  for (const p of draftOrder.slice(0, 10)) {
    const trade = p.notes ? ` (${p.notes})` : "";
    console.error(
      `  #${p.overall_pick} R${p.round}P${p.pick_in_round}: ${p.team_abbreviation}${trade}`,
    );
  }
} catch (err) {
  console.error(`Scraping failed: ${err.message}`);
  process.exit(1);
} finally {
  await browser.close();
}
