import { describe, it, expect } from "vitest";
import {
  CombineEntrySchema,
  CombineDataSchema,
  type CombineEntry,
  type CombineData,
} from "../../src/types/combine.js";

describe("CombineEntry", () => {
  it("can be constructed with all 11 measurable fields", () => {
    const entry: CombineEntry = {
      first_name: "Cam",
      last_name: "Ward",
      position: "QB",
      source: "pro_football_reference",
      year: 2026,
      forty_yard_dash: 4.72,
      bench_press: 18,
      vertical_jump: 32.0,
      broad_jump: 108,
      three_cone_drill: 7.05,
      twenty_yard_shuttle: 4.3,
      arm_length: 32.5,
      hand_size: 9.75,
      wingspan: 77.5,
      ten_yard_split: 1.65,
      twenty_yard_split: 2.72,
    };

    expect(entry.source).toBe("pro_football_reference");
    expect(entry.forty_yard_dash).toBe(4.72);
    expect(entry.bench_press).toBe(18);
    expect(entry.vertical_jump).toBe(32.0);
    expect(entry.broad_jump).toBe(108);
    expect(entry.three_cone_drill).toBe(7.05);
    expect(entry.twenty_yard_shuttle).toBe(4.3);
    expect(entry.arm_length).toBe(32.5);
    expect(entry.hand_size).toBe(9.75);
    expect(entry.wingspan).toBe(77.5);
    expect(entry.ten_yard_split).toBe(1.65);
    expect(entry.twenty_yard_split).toBe(2.72);
  });

  it("allows null for all measurable fields", () => {
    const entry: CombineEntry = {
      first_name: "Travis",
      last_name: "Hunter",
      position: "CB",
      source: "combine",
      year: 2026,
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

    expect(entry.forty_yard_dash).toBeNull();
    expect(entry.bench_press).toBeNull();
  });

  it("round-trips through JSON serialization", () => {
    const entry: CombineEntry = {
      first_name: "Travis",
      last_name: "Hunter",
      position: "CB",
      source: "pro_football_reference",
      year: 2026,
      forty_yard_dash: 4.38,
      bench_press: null,
      vertical_jump: 40.5,
      broad_jump: 130,
      three_cone_drill: null,
      twenty_yard_shuttle: null,
      arm_length: null,
      hand_size: null,
      wingspan: null,
      ten_yard_split: null,
      twenty_yard_split: null,
    };

    const json = JSON.stringify(entry);
    const deserialized: CombineEntry = JSON.parse(json);
    expect(deserialized).toEqual(entry);
  });

  it("validates correctly with Zod schema", () => {
    const valid = {
      first_name: "Cam",
      last_name: "Ward",
      position: "QB",
      source: "combine",
      year: 2026,
      forty_yard_dash: 4.72,
      bench_press: 18,
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

    const result = CombineEntrySchema.safeParse(valid);
    expect(result.success).toBe(true);
  });

  it("rejects invalid data with Zod schema", () => {
    const invalid = {
      first_name: "Cam",
      // missing last_name
      position: "QB",
      source: "combine",
      year: "not a number",
    };

    const result = CombineEntrySchema.safeParse(invalid);
    expect(result.success).toBe(false);
  });
});

describe("CombineData", () => {
  it("round-trips through JSON serialization", () => {
    const data: CombineData = {
      meta: {
        source: "pro_football_reference",
        description: "2026 NFL Combine results",
        year: 2026,
        generated_at: "2026-03-10",
        player_count: 1,
        entry_count: 1,
      },
      combine_results: [
        {
          first_name: "Cam",
          last_name: "Ward",
          position: "QB",
          source: "combine",
          year: 2026,
          forty_yard_dash: 4.72,
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
        },
      ],
    };

    const json = JSON.stringify(data);
    const deserialized = CombineDataSchema.parse(JSON.parse(json));
    expect(deserialized.meta.source).toBe("pro_football_reference");
    expect(deserialized.combine_results).toHaveLength(1);
    expect(deserialized.combine_results[0].first_name).toBe("Cam");
  });

  it("validates complete CombineData with Zod", () => {
    const data = {
      meta: {
        source: "merged",
        description: "test",
        year: 2026,
        generated_at: "2026-03-10",
        player_count: 0,
        entry_count: 0,
      },
      combine_results: [],
    };

    const result = CombineDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });
});
