import * as cheerio from "cheerio";
import type { RankingData, RankingEntry } from "../../types/rankings.js";
import { normalizePosition } from "../../shared/position-normalizer.js";
import { parseHeight, splitName } from "./helpers.js";

const KNOWN_POSITIONS = new Set([
  "QB", "RB", "HB", "FB", "WR", "TE", "OT", "T", "OG", "G", "IOL", "OL", "C",
  "DE", "EDGE", "DT", "DL", "NT", "LB", "OLB", "ILB", "MLB",
  "CB", "S", "SS", "FS", "DB", "SAF", "K", "P", "ATH",
]);

/**
 * Try multiple table selectors in order (matching Rust implementation).
 */
function findRows($: cheerio.CheerioAPI): cheerio.Cheerio<cheerio.Element> {
  const selectors = [
    "table.bpa tr",
    "table tr.pointed",
    "table[class*='draft'] tr",
  ];

  for (const sel of selectors) {
    const rows = $(sel);
    if (rows.length > 0) return rows;
  }

  return $([]);
}

function parseWeight(text: string): number | null {
  const trimmed = text.trim();
  if (!trimmed || trimmed === "-") return null;
  const num = parseInt(trimmed, 10);
  return isNaN(num) ? null : num;
}

export function parseDraftTekHtml(html: string, year: number): RankingData {
  const $ = cheerio.load(html);
  const rows = findRows($);
  const rankings: RankingEntry[] = [];

  rows.each((_, row) => {
    const cells = $(row).find("td");
    if (cells.length < 6) return;

    const rankText = $(cells[0]).text().trim();
    const nameText = $(cells[1]).text().trim();
    const school = $(cells[2]).text().trim();
    const posText = $(cells[3]).text().trim().toUpperCase();

    // Skip header rows — position must be a known football position
    if (!KNOWN_POSITIONS.has(posText)) return;

    const rank = parseInt(rankText, 10);
    if (isNaN(rank)) return;

    const [firstName, lastName] = splitName(nameText);
    const position = normalizePosition(posText);
    const heightInches = parseHeight($(cells[4]).text());
    const weightPounds = parseWeight($(cells[5]).text());

    rankings.push({
      rank,
      first_name: firstName,
      last_name: lastName,
      position,
      school,
      height_inches: heightInches,
      weight_pounds: weightPounds,
    });
  });

  return {
    meta: {
      version: "1.0.0",
      source: "drafttek",
      source_url: `https://www.drafttek.com/2026-NFL-Draft-Big-Board/Top-NFL-Draft-Prospects-2026-Page-1.asp`,
      draft_year: year,
      scraped_at: new Date().toISOString().slice(0, 10),
      total_prospects: rankings.length,
    },
    rankings,
  };
}
