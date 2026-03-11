import * as cheerio from "cheerio";
import type { RankingData, RankingEntry } from "../../types/rankings.js";
import { normalizePosition } from "../../shared/position-normalizer.js";

/**
 * Parse "N. Name, Position, School" from WalterFootball bold elements.
 * Format: "1. Travis Hunter, CB, Colorado"
 */
function parseProspectLine(text: string): RankingEntry | null {
  // Match: rank. name, position, school
  const match = text.match(/^(\d+)\.\s*(.+?),\s*([A-Za-z/]+),\s*(.+)$/);
  if (!match) return null;

  const rank = parseInt(match[1], 10);
  const fullName = match[2].trim();
  const rawPosition = match[3].trim();
  const school = match[4].trim();

  // Handle slash positions: take first one (e.g., "OT/G" → "OT")
  const position = normalizePosition(rawPosition.split("/")[0]);

  // Split name into first/last
  const nameParts = fullName.split(/\s+/);
  if (nameParts.length === 0) return null;

  const firstName = nameParts[0];
  const lastName = nameParts.length > 1 ? nameParts.slice(1).join(" ") : "";

  return {
    rank,
    first_name: firstName,
    last_name: lastName,
    position,
    school,
    height_inches: null,
    weight_pounds: null,
  };
}

export function parseWalterFootballHtml(html: string, year: number): RankingData {
  const $ = cheerio.load(html);
  const rankings: RankingEntry[] = [];

  // WalterFootball uses <b> and <strong> tags for prospect entries
  $("b, strong").each((_, el) => {
    // Get text content, stripping any <a> tags but keeping their text
    const text = $(el).text().trim();

    const entry = parseProspectLine(text);
    if (entry) {
      rankings.push(entry);
    }
  });

  return {
    meta: {
      version: "1.0.0",
      source: "walterfootball",
      source_url: `https://walterfootball.com/nfldraftbigboard${year}.php`,
      draft_year: year,
      scraped_at: new Date().toISOString().slice(0, 10),
      total_prospects: rankings.length,
    },
    rankings,
  };
}
