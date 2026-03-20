import * as cheerio from "cheerio";
import type { RankingData, RankingEntry } from "../../types/rankings.js";
import { normalizePosition } from "../../shared/position-normalizer.js";


export function parseWalterFootballHtml(html: string, year: number): RankingData {
  const $ = cheerio.load(html);
  const rankings: RankingEntry[] = [];

  // WalterFootball uses alternating <b> pairs (current 2026 structure):
  //   <b>1.</b>  <b>Jeremiyah Love,   RB, Notre Dame.</b>
  //   <b>2.</b>  <b>David Bailey,     DE, Texas Tech.</b>
  // Collect all <b> texts, then scan for a rank-only tag followed by a name/pos/school tag.
  const bTexts: string[] = [];
  $("b").each((_, el) => {
    bTexts.push($(el).text().trim());
  });

  for (let i = 0; i < bTexts.length - 1; i++) {
    const maybeRank = bTexts[i];
    const maybePlayer = bTexts[i + 1];

    // Rank tag: "1." or "10." etc.
    if (!/^\d+\.$/.test(maybeRank)) continue;
    const rank = parseInt(maybeRank, 10);
    if (isNaN(rank)) continue;

    // Player tag: "Name,   Position, School." (with variable whitespace)
    // Collapse whitespace first
    const normalized = maybePlayer.replace(/\s+/g, " ").replace(/\.$/, "").trim();
    // Match: "Full Name, Position, School"
    const m = normalized.match(/^(.+?),\s*([A-Za-z/]+),\s*(.+)$/);
    if (!m) continue;

    const fullName = m[1].trim();
    const rawPosition = m[2].trim();
    const school = m[3].trim();

    const nameParts = fullName.split(/\s+/);
    const firstName = nameParts[0] ?? "";
    const lastName = nameParts.length > 1 ? nameParts.slice(1).join(" ") : "";
    const position = normalizePosition(rawPosition.split("/")[0]);

    rankings.push({
      rank,
      first_name: firstName,
      last_name: lastName,
      position,
      school,
      height_inches: null,
      weight_pounds: null,
    });

    i++; // skip the player tag we just consumed
  }

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
