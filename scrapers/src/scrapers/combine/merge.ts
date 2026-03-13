import type { CombineData, CombineEntry } from "../../types/combine.js";
import { nameKey } from "../../shared/name-normalizer.js";

const BACKFILL_FIELDS: (keyof CombineEntry)[] = [
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

export interface MergeStats {
  primaryCount: number;
  matchedBackfills: number;
  uniqueAdded: number;
  fieldsBackfilled: number;
}

export function mergeCombineData(
  primary: CombineData,
  secondaries: CombineData[],
): CombineData {
  const year = primary.meta.year;
  const keyToIndex = new Map<string, number>();
  const merged: CombineEntry[] = [];

  // Add all primary entries
  for (const entry of primary.combine_results) {
    const key = nameKey(entry.first_name, entry.last_name);
    keyToIndex.set(key, merged.length);
    merged.push({ ...entry });
  }

  const stats: MergeStats = {
    primaryCount: primary.combine_results.length,
    matchedBackfills: 0,
    uniqueAdded: 0,
    fieldsBackfilled: 0,
  };

  // Process secondaries
  for (const secondary of secondaries) {
    for (const entry of secondary.combine_results) {
      const key = nameKey(entry.first_name, entry.last_name);

      if (keyToIndex.has(key)) {
        // Backfill missing fields
        const existing = merged[keyToIndex.get(key)!];
        let backfilled = false;
        for (const field of BACKFILL_FIELDS) {
          if (existing[field] == null && entry[field] != null) {
            (existing as Record<string, unknown>)[field] = entry[field];
            stats.fieldsBackfilled++;
            backfilled = true;
          }
        }
        if (backfilled) stats.matchedBackfills++;
      } else {
        // Unique to secondary
        keyToIndex.set(key, merged.length);
        merged.push({ ...entry });
        stats.uniqueAdded++;
      }
    }
  }

  // Log merge statistics
  if (secondaries.length > 0) {
    console.error("\nMerge statistics:");
    console.error(`  Primary entries: ${stats.primaryCount}`);
    console.error(`  Matched & backfilled: ${stats.matchedBackfills}`);
    console.error(`  Unique players added from secondaries: ${stats.uniqueAdded}`);
    console.error(`  Individual fields backfilled: ${stats.fieldsBackfilled}`);
    console.error(`  Total merged entries: ${merged.length}`);
  }

  return {
    meta: {
      source: "merged",
      description: `${year} NFL Combine results (merged from multiple sources)`,
      year,
      generated_at: new Date().toISOString(),
      player_count: merged.length,
      entry_count: merged.length,
    },
    combine_results: merged,
  };
}
