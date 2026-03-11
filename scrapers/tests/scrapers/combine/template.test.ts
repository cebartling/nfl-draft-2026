import { describe, it, expect } from "vitest";
import { generateTemplateCombine } from "../../../src/scrapers/combine/template.js";
import { CombineDataSchema } from "../../../src/types/combine.js";

describe("generateTemplateCombine", () => {
  it("generates entries for template prospects", () => {
    const data = generateTemplateCombine(2026);
    expect(data.combine_results.length).toBeGreaterThan(0);
  });

  it("sets meta source to template", () => {
    const data = generateTemplateCombine(2026);
    expect(data.meta.source).toBe("template");
    expect(data.meta.year).toBe(2026);
    expect(data.meta.player_count).toBe(data.combine_results.length);
  });

  it("all entries have null measurables", () => {
    const data = generateTemplateCombine(2026);
    for (const entry of data.combine_results) {
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
    }
  });

  it("all entries have non-empty names and positions", () => {
    const data = generateTemplateCombine(2026);
    for (const entry of data.combine_results) {
      expect(entry.first_name).not.toBe("");
      expect(entry.last_name).not.toBe("");
      expect(entry.position).not.toBe("");
    }
  });

  it("validates against Zod schema", () => {
    const data = generateTemplateCombine(2026);
    const result = CombineDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });
});
