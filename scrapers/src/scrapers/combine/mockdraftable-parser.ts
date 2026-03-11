import type { CombineData, CombineEntry } from "../../types/combine.js";
import { normalizePosition } from "../../shared/position-normalizer.js";
import { makeCombineEntry } from "../../shared/combine-helpers.js";
import { splitName } from "../../shared/name-normalizer.js";

/**
 * Extract the `window.INITIAL_STATE` JSON blob from page HTML.
 * Uses brace-matched extraction.
 */
export function extractInitialState(html: string): unknown {
  const marker = "window.INITIAL_STATE";
  const markerPos = html.indexOf(marker);
  if (markerPos === -1) {
    throw new Error("Could not find window.INITIAL_STATE in HTML. Try using --browser flag.");
  }

  const afterMarker = html.slice(markerPos);
  const braceStart = afterMarker.indexOf("{");
  if (braceStart === -1) {
    throw new Error("No opening brace after INITIAL_STATE");
  }

  const jsonStart = markerPos + braceStart;
  const rest = html.slice(jsonStart);

  // Brace-matched extraction (respects string literals)
  let depth = 0;
  let end = 0;
  let inString = false;
  for (let i = 0; i < rest.length; i++) {
    const ch = rest[i];
    if (inString) {
      if (ch === "\\" ) {
        i++; // skip escaped character
      } else if (ch === '"') {
        inString = false;
      }
    } else {
      if (ch === '"') {
        inString = true;
      } else if (ch === "{") {
        depth++;
      } else if (ch === "}") {
        depth--;
        if (depth === 0) {
          end = i + 1;
          break;
        }
      }
    }
  }

  if (end === 0) {
    throw new Error("Could not find matching closing brace for INITIAL_STATE JSON");
  }

  return JSON.parse(rest.slice(0, end));
}

/** Map measurable key ID to CombineEntry field. */
const KEY_MAP: Record<number, string> = {
  3: "wingspan",
  4: "arm_length",
  5: "hand_size",
  6: "ten_yard_split",
  7: "twenty_yard_split",
  8: "forty_yard_dash",
  9: "bench_press",
  10: "vertical_jump",
  11: "broad_jump",
  12: "three_cone_drill",
  13: "twenty_yard_shuttle",
};

/** Map string measurement names to CombineEntry field. */
const NAME_MAP: Record<string, string> = {
  "40 yard dash": "forty_yard_dash",
  "forty yard dash": "forty_yard_dash",
  "40-yard dash": "forty_yard_dash",
  "40yd": "forty_yard_dash",
  "bench press": "bench_press",
  bench: "bench_press",
  "vertical jump": "vertical_jump",
  vertical: "vertical_jump",
  vert: "vertical_jump",
  "broad jump": "broad_jump",
  "3 cone drill": "three_cone_drill",
  "three cone drill": "three_cone_drill",
  "3cone": "three_cone_drill",
  "3-cone drill": "three_cone_drill",
  "20 yard shuttle": "twenty_yard_shuttle",
  "twenty yard shuttle": "twenty_yard_shuttle",
  shuttle: "twenty_yard_shuttle",
  "short shuttle": "twenty_yard_shuttle",
  "20yd shuttle": "twenty_yard_shuttle",
  "arm length": "arm_length",
  arms: "arm_length",
  "hand size": "hand_size",
  hands: "hand_size",
  "hand length": "hand_size",
  wingspan: "wingspan",
  "wing span": "wingspan",
  "10 yard split": "ten_yard_split",
  "ten yard split": "ten_yard_split",
  "10yd": "ten_yard_split",
  "20 yard split": "twenty_yard_split",
  "twenty yard split": "twenty_yard_split",
  "20yd split": "twenty_yard_split",
};

const INT_FIELDS = new Set(["bench_press", "broad_jump"]);

function setMeasurement(entry: Record<string, string | number | null>, field: string, value: number): void {
  if (INT_FIELDS.has(field)) {
    entry[field] = Math.round(value);
  } else {
    entry[field] = value;
  }
}

/**
 * Parse INITIAL_STATE JSON into CombineData.
 * Handles two formats:
 * 1. Dict-keyed players (real Mockdraftable): players is object keyed by slug
 * 2. Array-format players: players is array with firstName/lastName
 */
export function parseInitialState(json: unknown, year: number): CombineData {
  if (json == null || typeof json !== "object") {
    return buildResult([], year);
  }

  const entries: CombineEntry[] = [];
  const obj = json as Record<string, unknown>;

  // Try dict-of-players format first
  const players = obj.players;
  if (players && typeof players === "object" && !Array.isArray(players)) {
    const values = Object.values(players) as Record<string, unknown>[];
    const isPlayerDict = values.length > 0 && (values[0].name != null || values[0].id != null);

    if (isPlayerDict) {
      for (const player of values) {
        const entry = parseDictPlayer(player, year);
        if (entry) entries.push(entry);
      }

      return buildResult(entries, year);
    }
  }

  // Fall back to array-of-players format
  const playersArray = findPlayersArray(json);
  if (playersArray) {
    for (const player of playersArray) {
      const entry = parseArrayPlayer(player, year);
      if (entry) entries.push(entry);
    }
  }

  return buildResult(entries, year);
}

function buildResult(entries: CombineEntry[], year: number): CombineData {
  return {
    meta: {
      source: "mockdraftable",
      description: `${year} NFL Combine results from Mockdraftable`,
      year,
      generated_at: new Date().toISOString(),
      player_count: entries.length,
      entry_count: entries.length,
    },
    combine_results: entries,
  };
}

function parseDictPlayer(player: Record<string, unknown>, year: number): CombineEntry | null {
  const fullName = (player.name as string) ?? "";
  if (!fullName) return null;

  const [firstName, lastName] = splitName(fullName);

  // Position from positions.primary or positions.all[0]
  let position = "";
  const positions = player.positions as Record<string, unknown> | undefined;
  if (positions) {
    const all = positions.all as string[] | undefined;
    position = (positions.primary as string) ?? all?.[0] ?? "";
  }
  position = normalizePosition(position);

  const entry = makeCombineEntry(firstName, lastName, position, year);

  // Parse measurements by numeric key
  if (Array.isArray(player.measurements)) {
    for (const m of player.measurements as Record<string, unknown>[]) {
      const key = m.measurableKey;
      const value = m.measurement;
      if (typeof key === "number" && typeof value === "number") {
        const field = KEY_MAP[key];
        if (field) setMeasurement(entry, field, value);
      }
    }
  }

  return entry;
}

function parseArrayPlayer(player: unknown, year: number): CombineEntry | null {
  if (player == null || typeof player !== "object") return null;
  const p = player as Record<string, unknown>;

  const firstName = ((p.firstName ?? p.first_name ?? "") as string).trim();
  const lastName = ((p.lastName ?? p.last_name ?? "") as string).trim();

  if (!firstName && !lastName) return null;

  // Position can be string or object
  let position = "";
  const pos = p.position;
  if (typeof pos === "string") {
    position = pos;
  } else if (pos && typeof pos === "object") {
    const posObj = pos as Record<string, unknown>;
    position = (posObj.abbreviation ?? posObj.name ?? "") as string;
  }
  position = normalizePosition(position);

  const entry = makeCombineEntry(firstName, lastName, position, year);

  // Parse measurements by name
  const measurements = (p.measurements ?? p.measurables) as unknown[] | undefined;
  if (Array.isArray(measurements)) {
    for (const m of measurements as Record<string, unknown>[]) {
      const name = m.measurementType ?? m.name ?? m.type;
      const value = m.measurement ?? m.value;

      let nameStr = "";
      if (typeof name === "string") {
        nameStr = name;
      } else if (name && typeof name === "object") {
        nameStr = ((name as Record<string, unknown>).name ?? "") as string;
      }

      if (nameStr && typeof value === "number") {
        const field = NAME_MAP[nameStr.toLowerCase()];
        if (field) setMeasurement(entry, field, value);
      }
    }
  }

  return entry;
}

function findPlayersArray(json: unknown, maxDepth: number = 10): unknown[] | null {
  if (maxDepth <= 0 || json == null || typeof json !== "object") return null;

  const obj = json as Record<string, unknown>;
  for (const key of ["players", "results", "searchResults", "prospects"]) {
    if (Array.isArray(obj[key]) && obj[key].length > 0 && looksLikePlayers(obj[key])) {
      return obj[key];
    }
  }

  for (const v of Object.values(obj)) {
    const found = findPlayersArray(v, maxDepth - 1);
    if (found) return found;
  }

  return null;
}

function looksLikePlayers(arr: unknown[]): boolean {
  const first = arr[0];
  if (first == null || typeof first !== "object") return false;
  const obj = first as Record<string, unknown>;
  return obj.firstName != null || obj.first_name != null ||
         obj.lastName != null || obj.last_name != null;
}
