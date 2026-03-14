import * as cheerio from "cheerio";
import type { CombineData, CombineEntry } from "../../types/combine.js";
import { normalizePosition } from "../../shared/position-normalizer.js";
import { splitName } from "../../shared/name-normalizer.js";

// --- API JSON parsing ---

export interface NflComCombineProfile {
  person: {
    firstName: string;
    lastName: string;
    displayName: string;
  };
  position?: string;
  fortyYardDash: { seconds: number } | null;
  benchPress: number | null;
  verticalJump: number | null;
  broadJump: number | null;
  threeConeDrill: number | null;
  twentyYardShuttle: number | null;
  armLength: number | null;
  handSize: number | null;
  wingspan: number | null;
  tenYardSplit: { seconds: number } | null;
  twentyYardSplit: { seconds: number } | null;
  [key: string]: unknown;
}

/**
 * Parse NFL.com API combine profiles into CombineData.
 * Skips profiles with missing or malformed person data.
 */
export function parseNflComApi(profiles: NflComCombineProfile[], year: number): CombineData {
  const entries: CombineEntry[] = [];

  for (const p of profiles) {
    // Guard against malformed API responses missing the person object
    if (!p.person || !p.person.firstName || !p.person.lastName) {
      continue;
    }

    const position = p.position ? normalizePosition(p.position) : "";

    entries.push({
      first_name: p.person.firstName,
      last_name: p.person.lastName,
      position,
      source: "combine",
      year,
      forty_yard_dash: p.fortyYardDash?.seconds ?? null,
      bench_press: p.benchPress != null ? Math.round(p.benchPress) : null,
      vertical_jump: p.verticalJump ?? null,
      broad_jump: p.broadJump != null ? Math.round(p.broadJump) : null,
      three_cone_drill: p.threeConeDrill ?? null,
      twenty_yard_shuttle: p.twentyYardShuttle ?? null,
      arm_length: p.armLength ?? null,
      hand_size: p.handSize ?? null,
      wingspan: p.wingspan ?? null,
      ten_yard_split: p.tenYardSplit?.seconds ?? null,
      twenty_yard_split: p.twentyYardSplit?.seconds ?? null,
    });
  }

  return {
    meta: {
      source: "nfl_com",
      description: `${year} NFL Combine results from NFL.com API`,
      year,
      generated_at: new Date().toISOString(),
      player_count: entries.length,
      entry_count: entries.length,
    },
    combine_results: entries,
  };
}

// --- HTML parsing (fallback for rendered pages) ---

/** Header text patterns mapped to CombineEntry field names. */
const HEADER_MAP: Record<string, keyof CombineEntry> = {
  "40yd": "forty_yard_dash",
  "40-yd": "forty_yard_dash",
  "40 yard": "forty_yard_dash",
  forty: "forty_yard_dash",
  bench: "bench_press",
  vert: "vertical_jump",
  vertical: "vertical_jump",
  broad: "broad_jump",
  "broad jump": "broad_jump",
  "3cone": "three_cone_drill",
  "3-cone": "three_cone_drill",
  cone: "three_cone_drill",
  shuttle: "twenty_yard_shuttle",
  "20yd shuttle": "twenty_yard_shuttle",
  arm: "arm_length",
  "arm length": "arm_length",
  hand: "hand_size",
  "hand size": "hand_size",
  wing: "wingspan",
  wingspan: "wingspan",
};

const INTEGER_FIELDS = new Set(["bench_press", "broad_jump"]);

function parseTime(s: string): number | null {
  const trimmed = s.trim();
  if (!trimmed || trimmed === "-" || trimmed === "—" || trimmed === "N/A" || trimmed === "DNS")
    return null;
  const num = parseFloat(trimmed);
  return isNaN(num) ? null : num;
}

function parseIntMeasurement(s: string): number | null {
  const trimmed = s.trim();
  if (!trimmed || trimmed === "-" || trimmed === "—" || trimmed === "N/A" || trimmed === "DNS")
    return null;
  const num = parseFloat(trimmed);
  return isNaN(num) ? null : Math.round(num);
}

function normalizeHeader(text: string): string {
  return text
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9 ]/g, "");
}

function findColumnMapping(
  headers: string[],
): { playerIdx: number; posIdx: number; fieldMap: Map<number, keyof CombineEntry> } {
  let playerIdx = -1;
  let posIdx = -1;
  const fieldMap = new Map<number, keyof CombineEntry>();

  for (let i = 0; i < headers.length; i++) {
    const norm = normalizeHeader(headers[i]);
    if (norm === "player" || norm === "name") {
      playerIdx = i;
      continue;
    }
    if (norm === "pos" || norm === "position") {
      posIdx = i;
      continue;
    }
    // Check against header map
    for (const [pattern, field] of Object.entries(HEADER_MAP)) {
      if (norm === pattern || norm.includes(pattern)) {
        fieldMap.set(i, field);
        break;
      }
    }
  }

  return { playerIdx, posIdx, fieldMap };
}

/**
 * Parse NFL.com combine tracker HTML into CombineData.
 * Uses a header-driven approach: reads column headers to determine field mapping,
 * making it resilient to table structure variations.
 */
export function parseNflComHtml(html: string, year: number): CombineData {
  const $ = cheerio.load(html);

  // Try multiple table selectors that NFL.com might use
  const selectors = [
    "table.nfl-o-table",
    ".combine-tracker table",
    "table[data-table='combine']",
    "#combine-results table",
    "table",
  ];

  let table: ReturnType<typeof $> | null = null;
  for (const selector of selectors) {
    const found = $(selector);
    if (found.length > 0) {
      table = found.first();
      break;
    }
  }

  if (!table) {
    throw new Error("Could not find combine results table in NFL.com HTML");
  }

  // Extract header texts
  const headers: string[] = [];
  table.find("thead tr:last-child th, thead tr:last-child td").each((_, el) => {
    headers.push($(el).text().trim());
  });

  // If no thead, try first row
  if (headers.length === 0) {
    table.find("tr:first-child th, tr:first-child td").each((_, el) => {
      headers.push($(el).text().trim());
    });
  }

  const { playerIdx, posIdx, fieldMap } = findColumnMapping(headers);

  const entries: CombineEntry[] = [];

  // Process data rows from tbody, or all tr after first if no tbody
  const rows =
    table.find("tbody tr").length > 0 ? table.find("tbody tr") : table.find("tr").slice(1);

  rows.each((_, row) => {
    const cells: string[] = [];
    $(row)
      .find("td, th")
      .each((_, cell) => {
        cells.push($(cell).text().trim());
      });

    if (cells.length === 0) return;

    // Get player name
    const playerText = playerIdx >= 0 ? cells[playerIdx] : cells[0];
    if (!playerText) return;

    const [firstName, lastName] = splitName(playerText);

    // Get position
    const posText = posIdx >= 0 ? cells[posIdx] : cells[1] || "";
    const position = normalizePosition(posText);

    // Build entry with all measurements null by default
    const entry: CombineEntry = {
      first_name: firstName,
      last_name: lastName,
      position,
      source: "combine",
      year,
      forty_yard_dash: null,
      bench_press: null,
      vertical_jump: null,
      broad_jump: null,
      three_cone_drill: null,
      twenty_yard_shuttle: null,
      arm_length: null,
      hand_size: null,
      wingspan: null,
      ten_yard_split: null,
      twenty_yard_split: null,
    };

    // Map cells to fields using header mapping
    for (const [colIdx, field] of fieldMap.entries()) {
      if (colIdx < cells.length) {
        const value = INTEGER_FIELDS.has(field)
          ? parseIntMeasurement(cells[colIdx])
          : parseTime(cells[colIdx]);
        (entry[field] as number | null) = value;
      }
    }

    entries.push(entry);
  });

  return {
    meta: {
      source: "nfl_com",
      description: `${year} NFL Combine results from NFL.com Combine Tracker`,
      year,
      generated_at: new Date().toISOString(),
      player_count: entries.length,
      entry_count: entries.length,
    },
    combine_results: entries,
  };
}
