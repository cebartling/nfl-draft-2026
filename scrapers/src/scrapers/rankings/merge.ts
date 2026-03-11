import type { RankingData, RankingEntry } from "../../types/rankings.js";
import { nameKey } from "../../shared/name-normalizer.js";

export function mergeRankings(primary: RankingData, secondaries: RankingData[]): RankingData {
  // Build map keyed by normalized name from primary
  const seen = new Map<string, RankingEntry>();
  const ordered: string[] = [];

  for (const entry of primary.rankings) {
    const key = nameKey(entry.first_name, entry.last_name);
    seen.set(key, { ...entry });
    ordered.push(key);
  }

  // Process each secondary: backfill height/weight, append unique
  for (const secondary of secondaries) {
    for (const entry of secondary.rankings) {
      const key = nameKey(entry.first_name, entry.last_name);

      if (seen.has(key)) {
        // Backfill missing fields
        const existing = seen.get(key)!;
        if (existing.height_inches == null && entry.height_inches != null) {
          existing.height_inches = entry.height_inches;
        }
        if (existing.weight_pounds == null && entry.weight_pounds != null) {
          existing.weight_pounds = entry.weight_pounds;
        }
      } else {
        // Unique to this secondary — append
        seen.set(key, { ...entry });
        ordered.push(key);
      }
    }
  }

  // Re-rank sequentially
  const rankings: RankingEntry[] = ordered.map((key, i) => ({
    ...seen.get(key)!,
    rank: i + 1,
  }));

  return {
    meta: {
      version: "1.0.0",
      source: "merged",
      source_url: primary.meta.source_url,
      draft_year: primary.meta.draft_year,
      scraped_at: new Date().toISOString().slice(0, 10),
      total_prospects: rankings.length,
    },
    rankings,
  };
}
