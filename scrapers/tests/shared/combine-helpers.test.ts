import { describe, it, expect } from "vitest";
import { makeCombineEntry } from "../../src/shared/combine-helpers.js";

describe("makeCombineEntry", () => {
  it("creates entry with all measurables null by default", () => {
    const entry = makeCombineEntry("Cam", "Ward", "QB", 2026);

    expect(entry.first_name).toBe("Cam");
    expect(entry.last_name).toBe("Ward");
    expect(entry.position).toBe("QB");
    expect(entry.source).toBe("combine");
    expect(entry.year).toBe(2026);
    expect(entry.forty_yard_dash).toBeNull();
    expect(entry.bench_press).toBeNull();
    expect(entry.vertical_jump).toBeNull();
    expect(entry.broad_jump).toBeNull();
    expect(entry.three_cone_drill).toBeNull();
    expect(entry.twenty_yard_shuttle).toBeNull();
    expect(entry.arm_length).toBeNull();
    expect(entry.hand_size).toBeNull();
    expect(entry.wingspan).toBeNull();
    expect(entry.ten_yard_split).toBeNull();
    expect(entry.twenty_yard_split).toBeNull();
  });

  it("allows overriding specific measurables", () => {
    const entry = makeCombineEntry("Cam", "Ward", "QB", 2026, {
      forty_yard_dash: 4.72,
      bench_press: 18,
    });

    expect(entry.forty_yard_dash).toBe(4.72);
    expect(entry.bench_press).toBe(18);
    expect(entry.vertical_jump).toBeNull();
  });
});
