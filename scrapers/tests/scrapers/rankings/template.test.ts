import { describe, it, expect } from "vitest";
import { generateTemplateRankings } from "../../../src/scrapers/rankings/template.js";
import { RankingDataSchema } from "../../../src/types/rankings.js";

describe("generateTemplateRankings", () => {
  it("generates at least 180 prospects", () => {
    const data = generateTemplateRankings(2026);
    expect(data.rankings.length).toBeGreaterThanOrEqual(180);
  });

  it("has sequential ranks starting at 1", () => {
    const data = generateTemplateRankings(2026);
    for (let i = 0; i < data.rankings.length; i++) {
      expect(data.rankings[i].rank).toBe(i + 1);
    }
  });

  it("includes a variety of positions", () => {
    const data = generateTemplateRankings(2026);
    const positions = new Set(data.rankings.map((r) => r.position));
    expect(positions.has("QB")).toBe(true);
    expect(positions.has("WR")).toBe(true);
    expect(positions.has("CB")).toBe(true);
    expect(positions.has("OT")).toBe(true);
    expect(positions.has("DT")).toBe(true);
    expect(positions.has("DE")).toBe(true);
  });

  it("has all non-empty fields for every prospect", () => {
    const data = generateTemplateRankings(2026);
    for (const entry of data.rankings) {
      expect(entry.first_name).not.toBe("");
      expect(entry.last_name).not.toBe("");
      expect(entry.position).not.toBe("");
      expect(entry.school).not.toBe("");
    }
  });

  it("sets meta source to template", () => {
    const data = generateTemplateRankings(2026);
    expect(data.meta.source).toBe("template");
    expect(data.meta.draft_year).toBe(2026);
    expect(data.meta.total_prospects).toBe(data.rankings.length);
  });

  it("validates against Zod schema", () => {
    const data = generateTemplateRankings(2026);
    const result = RankingDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });

  it("has null height and weight for all template entries", () => {
    const data = generateTemplateRankings(2026);
    for (const entry of data.rankings) {
      expect(entry.height_inches).toBeNull();
      expect(entry.weight_pounds).toBeNull();
    }
  });
});
