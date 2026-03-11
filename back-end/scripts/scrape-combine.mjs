#!/usr/bin/env node
/**
 * Playwright-based scraper for NFL Combine data.
 *
 * Fallback for when direct HTTP scraping is blocked (403, JS-rendered pages).
 *
 * Usage:
 *   node scrape-combine.mjs [--year 2026] [--output path] [--source pfr|mockdraftable]
 *
 * Output: JSON file in CombineData format compatible with seed-data combine_loader.
 */

import { writeFileSync } from "fs";
import { resolve, dirname, join } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// Resolve playwright from multiple locations
let chromium;
for (const candidate of [
  join(__dirname, "../../front-end/node_modules/playwright/index.mjs"),
  join(__dirname, "../../acceptance-tests/node_modules/playwright/index.mjs"),
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

const year = parseInt(getArg("--year", "2026"), 10);
const source = getArg("--source", "pfr");
const outputPath = resolve(
  getArg("--output", `data/combine_${year}.json`),
);

// Position normalization — must stay aligned with models.rs normalize_position()
// and seed-data/src/position_mapper.rs
const positionMap = {
  DE: "DE",
  EDGE: "DE",
  "EDGE/LB": "DE",
  "LB/EDGE": "DE",
  OLB: "LB",
  ILB: "LB",
  MLB: "LB",
  DL: "DT",
  NT: "DT",
  OG: "OG",
  G: "OG",
  IOL: "OG",
  OL: "OG",
  T: "OT",
  FS: "S",
  SS: "S",
  DB: "S",
  SAF: "S",
  FB: "RB",
  HB: "RB",
};

function normalizePosition(pos) {
  const upper = (pos || "").trim().toUpperCase();
  return positionMap[upper] ?? upper;
}

async function scrapePfr(page) {
  const url =
    `https://www.pro-football-reference.com/draft/${year}-combine.htm`;

  console.error(`Scraping PFR combine for ${year}...`);
  console.error(`URL: ${url}`);

  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 60000 });
  await page.waitForSelector("table#combine", { timeout: 30000 });

  const entries = await page.$$eval(
    "table#combine tbody tr:not(.thead)",
    (rows) => {
      return rows
        .filter((row) => {
          const cls = row.className || "";
          return !cls.includes("over_header") && !cls.includes("spacer");
        })
        .map((row) => {
          const playerTh = row.querySelector('th[data-stat="player"]');
          const fullName = playerTh?.textContent?.trim() ?? "";
          if (!fullName) return null;

          const parts = fullName.split(/\s+/);
          const firstName = parts[0] ?? "";
          const lastName = parts.slice(1).join(" ");

          const getStat = (stat) => {
            const td = row.querySelector(`td[data-stat="${stat}"]`);
            return td?.textContent?.trim() ?? "";
          };

          const parseFloat_ = (s) => {
            const v = parseFloat(s);
            return isNaN(v) ? null : v;
          };
          const parseInt_ = (s) => {
            const v = parseInt(s, 10);
            return isNaN(v) ? null : v;
          };

          return {
            first_name: firstName,
            last_name: lastName,
            position: getStat("pos"),
            forty_yard_dash: parseFloat_(getStat("forty_yd")),
            bench_press: parseInt_(getStat("bench_reps")),
            vertical_jump: parseFloat_(getStat("vertical")),
            broad_jump: parseInt_(getStat("broad_jump")),
            three_cone_drill: parseFloat_(getStat("cone")),
            twenty_yard_shuttle: parseFloat_(getStat("shuttle")),
            arm_length: parseFloat_(getStat("arm_length")),
            hand_size: parseFloat_(getStat("hand_size")),
            wingspan: parseFloat_(getStat("wingspan")),
            ten_yard_split: parseFloat_(getStat("ten_yd")),
            twenty_yard_split: parseFloat_(getStat("twenty_yd")),
          };
        })
        .filter(Boolean);
    },
  );

  return entries.map((e) => ({
    ...e,
    position: normalizePosition(e.position),
    source: "combine",
    year,
  }));
}

async function scrapeMockdraftable(page) {
  const url =
    `https://www.mockdraftable.com/search?year=${year}&beginYear=${year}&endYear=${year}&sort=name`;

  console.error(`Scraping Mockdraftable combine for ${year}...`);
  console.error(`URL: ${url}`);

  await page.goto(url, { waitUntil: "networkidle", timeout: 60000 });

  // Extract INITIAL_STATE
  const initialState = await page.evaluate(() => {
    return window.INITIAL_STATE;
  });

  if (!initialState) {
    throw new Error(
      "Could not find window.INITIAL_STATE on Mockdraftable page",
    );
  }

  // Measurable key ID mapping (numeric keys from Mockdraftable's real format)
  const measurableKeyMap = {
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

  // String-based measurement name mapping (legacy/fallback format)
  const measurementNameMap = {
    "40 yard dash": "forty_yard_dash",
    "forty yard dash": "forty_yard_dash",
    "bench press": "bench_press",
    bench: "bench_press",
    "vertical jump": "vertical_jump",
    vertical: "vertical_jump",
    "broad jump": "broad_jump",
    "3 cone drill": "three_cone_drill",
    "three cone drill": "three_cone_drill",
    "20 yard shuttle": "twenty_yard_shuttle",
    "short shuttle": "twenty_yard_shuttle",
    shuttle: "twenty_yard_shuttle",
    "arm length": "arm_length",
    arms: "arm_length",
    "hand size": "hand_size",
    hands: "hand_size",
    "hand length": "hand_size",
    wingspan: "wingspan",
    "10 yard split": "ten_yard_split",
    "20 yard split": "twenty_yard_split",
  };

  function makeEmptyEntry(firstName, lastName, position) {
    return {
      first_name: firstName,
      last_name: lastName,
      position: normalizePosition(position),
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
  }

  function setMeasurement(entry, field, value) {
    if (field && value != null) {
      entry[field] =
        field === "bench_press" || field === "broad_jump"
          ? Math.round(value)
          : value;
    }
  }

  // Try dict-keyed format first (real Mockdraftable site)
  function parseDictFormat(state) {
    // Players is a dict keyed by slug, e.g. { "cam-ward": { name: "Cam Ward", ... } }
    const playersObj = state.players;
    if (!playersObj || typeof playersObj !== "object" || Array.isArray(playersObj)) {
      return null;
    }

    // Build measurables name map from state.measurables (id -> name)
    const measurablesMap = {};
    if (state.measurables && typeof state.measurables === "object") {
      for (const [id, info] of Object.entries(state.measurables)) {
        if (info && info.name) {
          measurablesMap[parseInt(id, 10)] = info.name;
        }
      }
    }

    const entries = [];
    for (const player of Object.values(playersObj)) {
      const fullName = (player.name || "").trim();
      if (!fullName) continue;

      const parts = fullName.split(/\s+/);
      const firstName = parts[0] || "";
      const lastName = parts.slice(1).join(" ");

      const pos =
        typeof player.positions === "object" && player.positions?.primary
          ? typeof player.positions.primary === "string"
            ? player.positions.primary
            : player.positions.primary?.abbreviation || player.positions.primary?.name || ""
          : typeof player.position === "string"
            ? player.position
            : player.position?.abbreviation || "";

      const entry = makeEmptyEntry(firstName, lastName, pos);

      // Parse measurements from dict format
      const measurements = player.measurements || player.measurables;
      if (measurements && typeof measurements === "object" && !Array.isArray(measurements)) {
        for (const [key, m] of Object.entries(measurements)) {
          const measurableKey = m?.measurableKey ?? parseInt(key, 10);
          const value = m?.measurement ?? m?.value;
          const field = measurableKeyMap[measurableKey];
          setMeasurement(entry, field, value);
        }
      } else if (Array.isArray(measurements)) {
        for (const m of measurements) {
          const measurableKey = m?.measurableKey;
          if (measurableKey != null) {
            const field = measurableKeyMap[measurableKey];
            setMeasurement(entry, field, m?.measurement ?? m?.value);
          } else {
            const name = (m.measurementType || m.name || m.type?.name || "").toLowerCase();
            const field = measurementNameMap[name];
            setMeasurement(entry, field, m?.measurement ?? m?.value);
          }
        }
      }

      entries.push(entry);
    }

    return entries.length > 0 ? entries : null;
  }

  // Fallback: array-based format
  function parseArrayFormat(state) {
    function findPlayers(obj) {
      if (!obj || typeof obj !== "object") return null;
      for (const key of ["players", "results", "searchResults", "prospects"]) {
        if (Array.isArray(obj[key]) && obj[key].length > 0) {
          const first = obj[key][0];
          if (first.firstName || first.first_name || first.name) return obj[key];
        }
      }
      for (const val of Object.values(obj)) {
        const found = findPlayers(val);
        if (found) return found;
      }
      return null;
    }

    const players = findPlayers(state);
    if (!players) return null;

    return players
      .filter((p) => p.firstName || p.first_name || p.name)
      .map((p) => {
        let firstName, lastName;
        if (p.name) {
          const parts = p.name.trim().split(/\s+/);
          firstName = parts[0] || "";
          lastName = parts.slice(1).join(" ");
        } else {
          firstName = (p.firstName || p.first_name || "").trim();
          lastName = (p.lastName || p.last_name || "").trim();
        }

        const pos =
          typeof p.position === "string"
            ? p.position
            : p.position?.abbreviation || p.position?.name || "";

        const entry = makeEmptyEntry(firstName, lastName, pos);

        const measurements = p.measurements || p.measurables || [];
        if (Array.isArray(measurements)) {
          for (const m of measurements) {
            const name = (m.measurementType || m.name || m.type?.name || "").toLowerCase();
            const value = m.measurement ?? m.value;
            setMeasurement(entry, measurementNameMap[name], value);
          }
        }

        return entry;
      });
  }

  // Try dict format first (real site), fall back to array format
  let entries = parseDictFormat(initialState);
  if (!entries) {
    entries = parseArrayFormat(initialState);
  }
  if (!entries || entries.length === 0) {
    throw new Error("Could not find players in INITIAL_STATE (tried dict and array formats)");
  }

  return entries;
}

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();

try {
  let entries;
  if (source === "pfr") {
    entries = await scrapePfr(page);
  } else if (source === "mockdraftable") {
    entries = await scrapeMockdraftable(page);
  } else {
    throw new Error(`Unknown source: ${source}`);
  }

  const today = new Date().toISOString().slice(0, 10);
  const data = {
    meta: {
      source: source === "pfr" ? "pro_football_reference" : "mockdraftable",
      description: `${year} NFL Combine results from ${source} (browser)`,
      year,
      generated_at: today,
      player_count: entries.length,
      entry_count: entries.length,
    },
    combine_results: entries,
  };

  writeFileSync(outputPath, JSON.stringify(data, null, 2) + "\n");
  console.error(`Wrote ${entries.length} combine entries to ${outputPath}`);

  // Print first 5 for verification
  console.error("\nFirst 5 entries:");
  for (const e of entries.slice(0, 5)) {
    console.error(
      `  ${e.first_name} ${e.last_name} (${e.position}) - 40yd: ${e.forty_yard_dash ?? "N/A"}`,
    );
  }
} catch (err) {
  console.error(`Scraping failed: ${err.message}`);
  process.exit(1);
} finally {
  await browser.close();
}
