import type { CombineData, CombineEntry } from "../types/combine.js";
import { nameKey } from "./name-normalizer.js";

export interface ValidationResult {
  warnings: string[];
  errors: string[];
}

const MEASUREMENT_FIELDS: (keyof CombineEntry)[] = [
  "forty_yard_dash",
  "bench_press",
  "vertical_jump",
  "broad_jump",
  "three_cone_drill",
  "twenty_yard_shuttle",
  "arm_length",
  "hand_size",
  "wingspan",
  "ten_yard_split",
  "twenty_yard_split",
];

/**
 * Validate combine data quality.
 * Returns warnings (non-fatal issues) and errors (data too poor to use).
 */
export function validateCombineData(
  data: CombineData,
  options: { minPlayerCount?: number } = {},
): ValidationResult {
  const { minPlayerCount = 50 } = options;
  const warnings: string[] = [];
  const errors: string[] = [];

  // Check minimum player count
  if (data.combine_results.length < minPlayerCount) {
    errors.push(
      `Only ${data.combine_results.length} players found (minimum: ${minPlayerCount}). ` +
        "Data source may be incomplete or failing silently.",
    );
  }

  // Check percentage with all-null measurements
  const allNullCount = data.combine_results.filter((entry) =>
    MEASUREMENT_FIELDS.every((field) => entry[field] == null),
  ).length;

  if (data.combine_results.length > 0) {
    const allNullPct = (allNullCount / data.combine_results.length) * 100;
    if (allNullPct > 50) {
      warnings.push(
        `${allNullCount}/${data.combine_results.length} players (${allNullPct.toFixed(0)}%) ` +
          "have ALL measurements null. Data may be from a template fallback.",
      );
    }
  }

  // Check for duplicate names
  const seen = new Map<string, number>();
  for (const entry of data.combine_results) {
    const key = nameKey(entry.first_name, entry.last_name);
    seen.set(key, (seen.get(key) || 0) + 1);
  }
  const duplicates = [...seen.entries()].filter(([, count]) => count > 1);
  if (duplicates.length > 0) {
    warnings.push(
      `${duplicates.length} duplicate name(s) found: ${duplicates.map(([k, c]) => `${k} (${c}x)`).join(", ")}`,
    );
  }

  return { warnings, errors };
}
