import * as cheerio from "cheerio";
import type { RankingData, RankingEntry } from "../../types/rankings.js";
import { normalizePosition } from "../../shared/position-normalizer.js";
import { splitName } from "./helpers.js";

/**
 * Parse Tankathon big board HTML. Two strategies:
 * 1. CSS selectors for div.mock-row.nfl elements
 * 2. Embedded JSON fallback (__NEXT_DATA__ or similar)
 */
export function parseTankathonRankingsHtml(html: string, year: number): RankingData {
  const $ = cheerio.load(html);
  let rankings: RankingEntry[] = [];

  // Strategy 1: mock-row CSS selectors
  // Current DOM structure (2026):
  //   div.mock-row-pick-number  → rank
  //   div.mock-row-name         → full name
  //   div.mock-row-school-position → "POSITION | School" (e.g. "LB/EDGE | Ohio State")
  const mockRows = $("div.mock-row.nfl");
  if (mockRows.length > 0) {
    mockRows.each((_, row) => {
      const rankText = $(row).find(".mock-row-pick-number").text().trim();
      const name = $(row).find(".mock-row-name").text().trim();
      const posSchool = $(row).find(".mock-row-school-position").text().trim();

      const rank = parseInt(rankText, 10);
      if (isNaN(rank)) return;

      // "LB/EDGE | Ohio State " → ["LB/EDGE", "Ohio State"]
      const parts = posSchool.split("|");
      const rawPos = (parts[0] ?? "").trim().split("/")[0]; // take first of slash positions
      const school = (parts[1] ?? "").trim();

      const [firstName, lastName] = splitName(name);
      const position = normalizePosition(rawPos);

      rankings.push({
        rank,
        first_name: firstName,
        last_name: lastName,
        position,
        school,
        height_inches: null,
        weight_pounds: null,
      });
    });
  }

  // Strategy 2: Embedded JSON fallback
  if (rankings.length === 0) {
    rankings = tryExtractEmbeddedJson($, html, year);
  }

  return {
    meta: {
      version: "1.0.0",
      source: "tankathon",
      source_url: "https://www.tankathon.com/nfl/big_board",
      draft_year: year,
      scraped_at: new Date().toISOString().slice(0, 10),
      total_prospects: rankings.length,
    },
    rankings,
  };
}

interface EmbeddedPlayer {
  rank: number;
  name: string;
  pos: string;
  school: string;
}

function tryExtractEmbeddedJson(
  _$: cheerio.CheerioAPI,
  html: string,
  _year: number,
): RankingEntry[] {
  // Look for __NEXT_DATA__ pattern
  const nextDataMatch = html.match(/window\.__NEXT_DATA__\s*=\s*({[\s\S]*})\s*<\/script>/);
  if (!nextDataMatch) return [];

  try {
    const data = JSON.parse(nextDataMatch[1]);
    const players: EmbeddedPlayer[] = data?.props?.pageProps?.players ?? [];

    return players.map((p) => {
      const [firstName, lastName] = splitName(p.name);
      return {
        rank: p.rank,
        first_name: firstName,
        last_name: lastName,
        position: normalizePosition(p.pos),
        school: p.school,
        height_inches: null,
        weight_pounds: null,
      };
    });
  } catch {
    return [];
  }
}
