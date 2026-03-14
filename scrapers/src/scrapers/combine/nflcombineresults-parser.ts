import * as cheerio from "cheerio";
import type { CombineData, CombineEntry } from "../../types/combine.js";
import { normalizePosition } from "../../shared/position-normalizer.js";
import { splitName } from "../../shared/name-normalizer.js";

/** Header text patterns mapped to CombineEntry field names. */
const HEADER_MAP: Record<string, keyof CombineEntry> = {
  "40 yard": "forty_yard_dash",
  "forty": "forty_yard_dash",
  "bench press": "bench_press",
  "bench": "bench_press",
  "vert leap": "vertical_jump",
  "vertical": "vertical_jump",
  "broad jump": "broad_jump",
  "broad": "broad_jump",
  "3cone": "three_cone_drill",
  "3-cone": "three_cone_drill",
  "cone": "three_cone_drill",
  shuttle: "twenty_yard_shuttle",
  "20yd shuttle": "twenty_yard_shuttle",
};

/** Header substrings to skip (prevents false matches with measurement patterns). */
const SKIP_PATTERNS = ["60yd", "60 yard", "wonderlic", "college", "school", "height", "weight"];

const INTEGER_FIELDS = new Set(["bench_press", "broad_jump"]);

function parseTime(s: string): number | null {
  const trimmed = s.trim();
  if (!trimmed || trimmed === "-" || trimmed === "—" || trimmed === "N/A") return null;
  const num = parseFloat(trimmed);
  return isNaN(num) ? null : num;
}

function parseIntMeasurement(s: string): number | null {
  const trimmed = s.trim();
  if (!trimmed || trimmed === "-" || trimmed === "—" || trimmed === "N/A") return null;
  const num = parseFloat(trimmed);
  return isNaN(num) ? null : Math.round(num);
}

function normalizeHeader(text: string): string {
  return text
    .trim()
    .toLowerCase()
    .replace(/\(.*?\)/g, "")
    .replace(/[^a-z0-9 ]/g, "")
    .trim();
}

interface ColumnMap {
  yearIdx: number;
  nameIdx: number;
  posIdx: number;
  fieldMap: Map<number, keyof CombineEntry>;
}

function findColumnMapping(headers: string[]): ColumnMap {
  let yearIdx = -1;
  let nameIdx = -1;
  let posIdx = -1;
  const fieldMap = new Map<number, keyof CombineEntry>();

  for (let i = 0; i < headers.length; i++) {
    const norm = normalizeHeader(headers[i]);
    if (norm === "year") {
      yearIdx = i;
      continue;
    }
    if (norm === "name" || norm === "player") {
      nameIdx = i;
      continue;
    }
    if (norm === "pos" || norm === "position") {
      posIdx = i;
      continue;
    }
    // Skip headers that are known non-measurement columns
    if (SKIP_PATTERNS.some((p) => norm.includes(p))) continue;

    // Check against header map
    for (const [pattern, field] of Object.entries(HEADER_MAP)) {
      if (norm === pattern || norm.includes(pattern)) {
        fieldMap.set(i, field);
        break;
      }
    }
  }

  return { yearIdx, nameIdx, posIdx, fieldMap };
}

/**
 * Parse nflcombineresults.com HTML table into CombineData.
 * Filters rows by year column if present.
 */
export function parseNflCombineResultsHtml(html: string, year: number): CombineData {
  const $ = cheerio.load(html);

  const selectors = ["table#datatable", "table.datatable", "table"];
  let table: ReturnType<typeof $> | null = null;
  for (const selector of selectors) {
    const found = $(selector);
    if (found.length > 0) {
      table = found.first();
      break;
    }
  }

  if (!table) {
    throw new Error("Could not find data table in nflcombineresults.com HTML");
  }

  // Extract header texts
  const headers: string[] = [];
  table.find("thead tr:last-child th, thead tr:last-child td").each((_, el) => {
    headers.push($(el).text().trim());
  });

  if (headers.length === 0) {
    table.find("tr:first-child th, tr:first-child td").each((_, el) => {
      headers.push($(el).text().trim());
    });
  }

  const { yearIdx, nameIdx, posIdx, fieldMap } = findColumnMapping(headers);

  const entries: CombineEntry[] = [];

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

    // Filter by year if Year column exists
    if (yearIdx >= 0) {
      const rowYear = parseInt(cells[yearIdx], 10);
      if (isNaN(rowYear) || rowYear !== year) return;
    }

    // Get player name from the detected Name column.
    // Fallback: if no Name header detected, assume column 1 when Year column exists
    // (Year is column 0), otherwise column 0. This matches the known
    // nflcombineresults.com layout: [Year, Name, College, POS, ...drills].
    const nameColIdx = nameIdx >= 0 ? nameIdx : yearIdx >= 0 ? 1 : 0;
    const playerText = cells[nameColIdx];
    if (!playerText) return;

    const [firstName, lastName] = splitName(playerText);

    // Get position — only use detected column, don't guess
    const posText = posIdx >= 0 && posIdx < cells.length ? cells[posIdx] : "";
    const position = normalizePosition(posText);

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
      source: "nflcombineresults",
      description: `${year} NFL Combine results from nflcombineresults.com`,
      year,
      generated_at: new Date().toISOString(),
      player_count: entries.length,
      entry_count: entries.length,
    },
    combine_results: entries,
  };
}
