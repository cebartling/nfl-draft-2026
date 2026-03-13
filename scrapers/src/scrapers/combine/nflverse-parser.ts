import type { CombineData } from "../../types/combine.js";
import { makeCombineEntry } from "../../shared/combine-helpers.js";
import { splitName } from "../../shared/name-normalizer.js";
import { normalizePosition } from "../../shared/position-normalizer.js";

function parseFloat_(s: string): number | null {
  if (!s || s.trim() === "") return null;
  const num = parseFloat(s.trim());
  return isNaN(num) ? null : num;
}

function parseInt_(s: string): number | null {
  if (!s || s.trim() === "") return null;
  const num = parseFloat(s.trim());
  return isNaN(num) ? null : Math.round(num);
}

/**
 * Parse nflverse combine CSV into CombineData.
 *
 * CSV columns: season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,
 *              player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
 *
 * Maps: forty→forty_yard_dash, bench→bench_press, vertical→vertical_jump,
 *       broad_jump→broad_jump, cone→three_cone_drill, shuttle→twenty_yard_shuttle
 *
 * Fields not in nflverse (always null): arm_length, hand_size, wingspan,
 *                                        ten_yard_split, twenty_yard_split
 */
export function parseNflverseCsv(csvText: string, year: number): CombineData {
  const lines = csvText.trim().split("\n");
  if (lines.length < 2) {
    return makeResult([], year);
  }

  const headers = lines[0].split(",");
  const colIndex = new Map<string, number>();
  headers.forEach((h, i) => colIndex.set(h.trim(), i));

  const col = (row: string[], name: string): string => {
    const idx = colIndex.get(name);
    return idx !== undefined ? (row[idx] || "") : "";
  };

  const entries = [];

  for (let i = 1; i < lines.length; i++) {
    const line = lines[i].trim();
    if (!line) continue;

    const row = line.split(",");
    const season = parseInt(col(row, "season"), 10);
    if (season !== year) continue;

    const playerName = col(row, "player_name").trim();
    if (!playerName) continue;

    const [firstName, lastName] = splitName(playerName);
    const position = normalizePosition(col(row, "pos"));

    entries.push(
      makeCombineEntry(firstName, lastName, position, year, {
        forty_yard_dash: parseFloat_(col(row, "forty")),
        bench_press: parseInt_(col(row, "bench")),
        vertical_jump: parseFloat_(col(row, "vertical")),
        broad_jump: parseInt_(col(row, "broad_jump")),
        three_cone_drill: parseFloat_(col(row, "cone")),
        twenty_yard_shuttle: parseFloat_(col(row, "shuttle")),
      }),
    );
  }

  return makeResult(entries, year);
}

function makeResult(entries: ReturnType<typeof makeCombineEntry>[], year: number): CombineData {
  return {
    meta: {
      source: "nflverse",
      description: `${year} NFL Combine results from nflverse`,
      year,
      generated_at: new Date().toISOString(),
      player_count: entries.length,
      entry_count: entries.length,
    },
    combine_results: entries,
  };
}
