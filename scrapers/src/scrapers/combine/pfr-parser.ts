import * as cheerio from "cheerio";
import type { CombineData, CombineEntry } from "../../types/combine.js";
import { normalizePosition } from "../../shared/position-normalizer.js";

function parseTime(s: string): number | null {
  const trimmed = s.trim();
  if (!trimmed || trimmed === "-") return null;
  const num = parseFloat(trimmed);
  return isNaN(num) ? null : num;
}

function parseIntMeasurement(s: string): number | null {
  const trimmed = s.trim();
  if (!trimmed || trimmed === "-") return null;
  const num = parseInt(trimmed, 10);
  return isNaN(num) ? null : num;
}

function splitName(fullName: string): [string, string] {
  const parts = fullName.trim().split(/\s+/);
  if (parts.length <= 1) return [fullName.trim(), ""];
  return [parts[0], parts.slice(1).join(" ")];
}

/**
 * Parse PFR combine HTML into CombineData.
 * PFR uses `<table id="combine">` with `data-stat` attributes on cells.
 */
export function parsePfrHtml(html: string, year: number): CombineData {
  const $ = cheerio.load(html);

  const table = $("table#combine");
  if (table.length === 0) {
    throw new Error("Could not find table#combine in PFR HTML");
  }

  const entries: CombineEntry[] = [];

  table.find("tbody tr:not(.thead)").each((_, row) => {
    const $row = $(row);
    const rowClass = $row.attr("class") || "";
    if (rowClass.includes("over_header") || rowClass.includes("spacer")) return;

    // Player name is in <th data-stat="player">
    const playerTh = $row.find('th[data-stat="player"]');
    const playerName = playerTh.text().trim();
    if (!playerName) return;

    const [firstName, lastName] = splitName(playerName);

    // Collect td cells by data-stat
    const cells: Record<string, string> = {};
    $row.find("td").each((_, td) => {
      const stat = $(td).attr("data-stat");
      if (stat) cells[stat] = $(td).text().trim();
    });

    const position = normalizePosition(cells["pos"] || "");

    entries.push({
      first_name: firstName,
      last_name: lastName,
      position,
      source: "combine",
      year,
      forty_yard_dash: parseTime(cells["forty_yd"] || ""),
      bench_press: parseIntMeasurement(cells["bench_reps"] || ""),
      vertical_jump: parseTime(cells["vertical"] || ""),
      broad_jump: parseIntMeasurement(cells["broad_jump"] || ""),
      three_cone_drill: parseTime(cells["cone"] || ""),
      twenty_yard_shuttle: parseTime(cells["shuttle"] || ""),
      arm_length: parseTime(cells["arm_length"] || ""),
      hand_size: parseTime(cells["hand_size"] || ""),
      wingspan: parseTime(cells["wingspan"] || ""),
      ten_yard_split: parseTime(cells["ten_yd"] || ""),
      twenty_yard_split: parseTime(cells["twenty_yd"] || ""),
    });
  });

  return {
    meta: {
      source: "pro_football_reference",
      description: `${year} NFL Combine results from Pro Football Reference`,
      year,
      generated_at: new Date().toISOString(),
      player_count: entries.length,
      entry_count: entries.length,
    },
    combine_results: entries,
  };
}
