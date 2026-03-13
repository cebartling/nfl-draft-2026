import { describe, it, expect } from "vitest";
import { validateCombineData } from "../../src/shared/combine-validator.js";
import { makeCombineEntry } from "../../src/shared/combine-helpers.js";
import type { CombineData, CombineMeta } from "../../src/types/combine.js";

function makeMeta(count: number): CombineMeta {
  return {
    source: "test",
    description: "test",
    year: 2026,
    generated_at: "2026-03-10",
    player_count: count,
    entry_count: count,
  };
}

function makeData(entries: ReturnType<typeof makeCombineEntry>[]): CombineData {
  return { meta: makeMeta(entries.length), combine_results: entries };
}

describe("validateCombineData", () => {
  it("returns error when player count is below minimum", () => {
    const entries = Array.from({ length: 10 }, (_, i) =>
      makeCombineEntry(`Player${i}`, "Test", "QB", 2026, { forty_yard_dash: 4.5 }),
    );
    const result = validateCombineData(makeData(entries));
    expect(result.errors.length).toBe(1);
    expect(result.errors[0]).toContain("10 players");
    expect(result.errors[0]).toContain("minimum: 50");
  });

  it("passes when player count meets minimum", () => {
    const entries = Array.from({ length: 55 }, (_, i) =>
      makeCombineEntry(`Player${i}`, "Test", "QB", 2026, { forty_yard_dash: 4.5 }),
    );
    const result = validateCombineData(makeData(entries));
    expect(result.errors.length).toBe(0);
  });

  it("allows custom minimum player count", () => {
    const entries = Array.from({ length: 5 }, (_, i) =>
      makeCombineEntry(`Player${i}`, "Test", "QB", 2026, { forty_yard_dash: 4.5 }),
    );
    const result = validateCombineData(makeData(entries), { minPlayerCount: 3 });
    expect(result.errors.length).toBe(0);
  });

  it("warns when >50% of players have all-null measurements", () => {
    const entries = [
      makeCombineEntry("Player1", "A", "QB", 2026, { forty_yard_dash: 4.5 }),
      makeCombineEntry("Player2", "B", "RB", 2026),
      makeCombineEntry("Player3", "C", "WR", 2026),
      makeCombineEntry("Player4", "D", "TE", 2026),
    ];
    const result = validateCombineData(makeData(entries), { minPlayerCount: 1 });
    expect(result.warnings.some((w) => w.includes("ALL measurements null"))).toBe(true);
  });

  it("does not warn when <=50% have all-null measurements", () => {
    const entries = [
      makeCombineEntry("Player1", "A", "QB", 2026, { forty_yard_dash: 4.5 }),
      makeCombineEntry("Player2", "B", "RB", 2026, { bench_press: 20 }),
      makeCombineEntry("Player3", "C", "WR", 2026),
    ];
    const result = validateCombineData(makeData(entries), { minPlayerCount: 1 });
    expect(result.warnings.some((w) => w.includes("ALL measurements null"))).toBe(false);
  });

  it("warns on duplicate names", () => {
    const entries = [
      makeCombineEntry("Cam", "Ward", "QB", 2026, { forty_yard_dash: 4.72 }),
      makeCombineEntry("Cam", "Ward", "QB", 2026, { forty_yard_dash: 4.75 }),
      makeCombineEntry("Travis", "Hunter", "CB", 2026),
    ];
    const result = validateCombineData(makeData(entries), { minPlayerCount: 1 });
    expect(result.warnings.some((w) => w.includes("duplicate"))).toBe(true);
    expect(result.warnings.some((w) => w.includes("cam ward"))).toBe(true);
  });

  it("returns no warnings/errors for valid data", () => {
    const entries = Array.from({ length: 60 }, (_, i) =>
      makeCombineEntry(`Player${i}`, "Test", "QB", 2026, { forty_yard_dash: 4.5 + i * 0.01 }),
    );
    const result = validateCombineData(makeData(entries));
    expect(result.errors.length).toBe(0);
    expect(result.warnings.length).toBe(0);
  });

  it("handles empty data without crashing", () => {
    const result = validateCombineData(makeData([]));
    expect(result.errors.length).toBe(1); // below minimum
    expect(result.warnings.length).toBe(0); // no null % warning for empty
  });
});
