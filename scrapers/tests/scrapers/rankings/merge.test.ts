import { describe, it, expect } from "vitest";
import { mergeRankings } from "../../../src/scrapers/rankings/merge.js";
import type { RankingData } from "../../../src/types/rankings.js";
import { RankingDataSchema } from "../../../src/types/rankings.js";

function makeData(
  source: string,
  entries: { rank: number; first: string; last: string; pos: string; school: string; height?: number | null; weight?: number | null }[],
): RankingData {
  return {
    meta: {
      version: "1.0.0",
      source,
      source_url: "test",
      draft_year: 2026,
      scraped_at: "2026-03-10",
      total_prospects: entries.length,
    },
    rankings: entries.map((e) => ({
      rank: e.rank,
      first_name: e.first,
      last_name: e.last,
      position: e.pos,
      school: e.school,
      height_inches: e.height ?? null,
      weight_pounds: e.weight ?? null,
    })),
  };
}

describe("mergeRankings", () => {
  it("returns primary when no secondaries", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado" },
    ]);
    const result = mergeRankings(primary, []);
    expect(result.rankings.length).toBe(1);
    expect(result.rankings[0].first_name).toBe("Travis");
  });

  it("appends unique prospects from secondary", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado" },
    ]);
    const secondary = makeData("drafttek", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado" },
      { rank: 2, first: "Shedeur", last: "Sanders", pos: "QB", school: "Colorado" },
    ]);
    const result = mergeRankings(primary, [secondary]);
    expect(result.rankings.length).toBe(2);
    expect(result.rankings[1].first_name).toBe("Shedeur");
  });

  it("backfills height from secondary", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado" },
    ]);
    const secondary = makeData("drafttek", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado", height: 73, weight: 185 },
    ]);
    const result = mergeRankings(primary, [secondary]);
    expect(result.rankings[0].height_inches).toBe(73);
    expect(result.rankings[0].weight_pounds).toBe(185);
  });

  it("does not overwrite existing height/weight", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado", height: 73, weight: 185 },
    ]);
    const secondary = makeData("drafttek", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado", height: 74, weight: 190 },
    ]);
    const result = mergeRankings(primary, [secondary]);
    expect(result.rankings[0].height_inches).toBe(73);
    expect(result.rankings[0].weight_pounds).toBe(185);
  });

  it("deduplicates by normalized name (case-insensitive)", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado" },
    ]);
    const secondary = makeData("drafttek", [
      { rank: 1, first: "travis", last: "hunter", pos: "CB", school: "Colorado" },
    ]);
    const result = mergeRankings(primary, [secondary]);
    expect(result.rankings.length).toBe(1);
  });

  it("handles Jr./III suffix normalization for dedup", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "Rueben", last: "Bain Jr.", pos: "EDGE", school: "Miami" },
    ]);
    const secondary = makeData("drafttek", [
      { rank: 1, first: "Rueben", last: "Bain", pos: "DE", school: "Miami" },
    ]);
    const result = mergeRankings(primary, [secondary]);
    expect(result.rankings.length).toBe(1);
  });

  it("cross-fills from multiple secondaries", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado" },
    ]);
    const sec1 = makeData("drafttek", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado", height: 73 },
    ]);
    const sec2 = makeData("walterfootball", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado", weight: 185 },
    ]);
    const result = mergeRankings(primary, [sec1, sec2]);
    expect(result.rankings[0].height_inches).toBe(73);
    expect(result.rankings[0].weight_pounds).toBe(185);
  });

  it("re-ranks after merge", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado" },
    ]);
    const secondary = makeData("drafttek", [
      { rank: 5, first: "Shedeur", last: "Sanders", pos: "QB", school: "Colorado" },
    ]);
    const result = mergeRankings(primary, [secondary]);
    expect(result.rankings[0].rank).toBe(1);
    expect(result.rankings[1].rank).toBe(2);
  });

  it("sets meta source to merged", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado" },
    ]);
    const result = mergeRankings(primary, []);
    expect(result.meta.source).toBe("merged");
  });

  it("validates against Zod schema", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "Travis", last: "Hunter", pos: "CB", school: "Colorado" },
    ]);
    const secondary = makeData("drafttek", [
      { rank: 2, first: "Shedeur", last: "Sanders", pos: "QB", school: "Colorado", height: 74, weight: 215 },
    ]);
    const result = mergeRankings(primary, [secondary]);
    const parsed = RankingDataSchema.safeParse(result);
    expect(parsed.success).toBe(true);
  });

  it("handles Unicode quotes in names for dedup", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "D\u2019Angelo", last: "Ponds", pos: "CB", school: "Indiana" },
    ]);
    const secondary = makeData("drafttek", [
      { rank: 1, first: "D'Angelo", last: "Ponds", pos: "CB", school: "Indiana" },
    ]);
    const result = mergeRankings(primary, [secondary]);
    expect(result.rankings.length).toBe(1);
  });

  it("handles periods in names for dedup (C.J. vs CJ)", () => {
    const primary = makeData("tankathon", [
      { rank: 1, first: "C.J.", last: "Allen", pos: "LB", school: "Georgia" },
    ]);
    const secondary = makeData("drafttek", [
      { rank: 1, first: "CJ", last: "Allen", pos: "LB", school: "Georgia" },
    ]);
    const result = mergeRankings(primary, [secondary]);
    expect(result.rankings.length).toBe(1);
  });
});
