import { describe, it, expect } from "vitest";
import { mergeCombineData } from "../../../src/scrapers/combine/merge.js";
import { makeCombineEntry } from "../../../src/shared/combine-helpers.js";
import type { CombineData, CombineMeta } from "../../../src/types/combine.js";
import { CombineDataSchema } from "../../../src/types/combine.js";

function makeMeta(source: string): CombineMeta {
  return {
    source,
    description: "test",
    year: 2026,
    generated_at: "2026-03-10",
    player_count: 0,
    entry_count: 0,
  };
}

function makeData(source: string, entries: ReturnType<typeof makeCombineEntry>[]): CombineData {
  return { meta: makeMeta(source), combine_results: entries };
}

describe("mergeCombineData", () => {
  it("backfills missing fields from secondary", () => {
    const pfr = makeCombineEntry("Cam", "Ward", "QB", 2026, { forty_yard_dash: 4.72 });
    const md = makeCombineEntry("Cam", "Ward", "QB", 2026, { forty_yard_dash: 4.75, arm_length: 32.5 });

    const result = mergeCombineData(makeData("pfr", [pfr]), [makeData("md", [md])]);
    expect(result.combine_results.length).toBe(1);
    expect(result.combine_results[0].forty_yard_dash).toBe(4.72); // kept primary
    expect(result.combine_results[0].arm_length).toBe(32.5); // backfilled
  });

  it("appends unique players from secondary", () => {
    const primary = makeData("pfr", [makeCombineEntry("Cam", "Ward", "QB", 2026)]);
    const secondary = makeData("md", [makeCombineEntry("Travis", "Hunter", "CB", 2026)]);
    const result = mergeCombineData(primary, [secondary]);
    expect(result.combine_results.length).toBe(2);
    expect(result.combine_results[1].first_name).toBe("Travis");
  });

  it("deduplicates by normalized name (case-insensitive)", () => {
    const primary = makeData("pfr", [makeCombineEntry("CAM", "WARD", "QB", 2026)]);
    const secondary = makeData("md", [makeCombineEntry("cam", "ward", "QB", 2026)]);
    const result = mergeCombineData(primary, [secondary]);
    expect(result.combine_results.length).toBe(1);
  });

  it("handles Jr./III suffix normalization", () => {
    const primary = makeData("pfr", [
      makeCombineEntry("Fernando", "Carmona Jr.", "OT", 2026, { forty_yard_dash: 5.15 }),
    ]);
    const secondary = makeData("md", [
      makeCombineEntry("Fernando", "Carmona", "OT", 2026, { arm_length: 34.0 }),
    ]);
    const result = mergeCombineData(primary, [secondary]);
    expect(result.combine_results.length).toBe(1);
    expect(result.combine_results[0].arm_length).toBe(34.0);
  });

  it("handles C.J. vs CJ dedup", () => {
    const primary = makeData("pfr", [makeCombineEntry("C.J.", "Stroud", "QB", 2026)]);
    const secondary = makeData("md", [makeCombineEntry("CJ", "Stroud", "QB", 2026)]);
    const result = mergeCombineData(primary, [secondary]);
    expect(result.combine_results.length).toBe(1);
  });

  it("handles Will Lee III vs Will Lee dedup", () => {
    const primary = makeData("pfr", [makeCombineEntry("Will", "Lee III", "CB", 2026)]);
    const secondary = makeData("md", [makeCombineEntry("Will", "Lee", "CB", 2026)]);
    const result = mergeCombineData(primary, [secondary]);
    expect(result.combine_results.length).toBe(1);
  });

  it("returns primary when no secondaries", () => {
    const primary = makeData("pfr", [makeCombineEntry("Cam", "Ward", "QB", 2026)]);
    const result = mergeCombineData(primary, []);
    expect(result.combine_results.length).toBe(1);
  });

  it("sets meta source to merged", () => {
    const primary = makeData("pfr", [makeCombineEntry("Cam", "Ward", "QB", 2026)]);
    const result = mergeCombineData(primary, []);
    expect(result.meta.source).toBe("merged");
    expect(result.meta.year).toBe(2026);
  });

  it("does not overwrite existing fields", () => {
    const pfr = makeCombineEntry("Cam", "Ward", "QB", 2026, {
      forty_yard_dash: 4.72,
      bench_press: 18,
      arm_length: 32.5,
    });
    const md = makeCombineEntry("Cam", "Ward", "QB", 2026, {
      forty_yard_dash: 4.75, // different — should NOT overwrite
      bench_press: 20, // different — should NOT overwrite
      arm_length: 33.0, // different — should NOT overwrite
      hand_size: 9.75, // new — should backfill
    });

    const result = mergeCombineData(makeData("pfr", [pfr]), [makeData("md", [md])]);
    const merged = result.combine_results[0];
    expect(merged.forty_yard_dash).toBe(4.72);
    expect(merged.bench_press).toBe(18);
    expect(merged.arm_length).toBe(32.5);
    expect(merged.hand_size).toBe(9.75);
  });

  it("handles apostrophe name variations in merge", () => {
    const primary = makeData("pfr", [
      makeCombineEntry("D'Angelo", "Smith", "WR", 2026, { forty_yard_dash: 4.42 }),
    ]);
    const secondary = makeData("md", [
      makeCombineEntry("DAngelo", "Smith", "WR", 2026, { arm_length: 32.0 }),
    ]);
    const result = mergeCombineData(primary, [secondary]);
    expect(result.combine_results.length).toBe(1);
    expect(result.combine_results[0].arm_length).toBe(32.0);
  });

  it("handles hyphenated name variations in merge", () => {
    const primary = makeData("nflverse", [
      makeCombineEntry("Quinshon", "Judkins-McNeil", "RB", 2026, { forty_yard_dash: 4.55 }),
    ]);
    const secondary = makeData("pfr", [
      makeCombineEntry("Quinshon", "JudkinsMcNeil", "RB", 2026, { arm_length: 33.5 }),
    ]);
    const result = mergeCombineData(primary, [secondary]);
    expect(result.combine_results.length).toBe(1);
    expect(result.combine_results[0].arm_length).toBe(33.5);
  });

  it("validates against Zod schema", () => {
    const primary = makeData("pfr", [
      makeCombineEntry("Cam", "Ward", "QB", 2026, { forty_yard_dash: 4.72 }),
    ]);
    const secondary = makeData("md", [
      makeCombineEntry("Travis", "Hunter", "CB", 2026, { forty_yard_dash: 4.38 }),
    ]);
    const result = mergeCombineData(primary, [secondary]);
    const parsed = CombineDataSchema.safeParse(result);
    expect(parsed.success).toBe(true);
  });
});
