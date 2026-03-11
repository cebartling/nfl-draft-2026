import { describe, it, expect } from "vitest";
import { generateTemplateDraftOrder } from "../../../src/scrapers/draft-order/template.js";
import { VALID_TEAM_ABBREVIATIONS } from "../../../src/shared/team-abbreviations.js";

describe("generateTemplateDraftOrder", () => {
  it("has correct structure", () => {
    const data = generateTemplateDraftOrder(2026);

    expect(data.meta.draft_year).toBe(2026);
    expect(data.meta.total_rounds).toBe(7);
    expect(data.meta.source).toBe("template");
    expect(data.draft_order.length).toBeGreaterThan(224); // 32*7 + comp picks
    expect(data.meta.total_picks).toBe(data.draft_order.length);
  });

  it("starts with overall pick 1", () => {
    const data = generateTemplateDraftOrder(2026);
    expect(data.draft_order[0].overall_pick).toBe(1);
    expect(data.draft_order[0].round).toBe(1);
    expect(data.draft_order[0].pick_in_round).toBe(1);
  });

  it("has last pick equal to total", () => {
    const data = generateTemplateDraftOrder(2026);
    const last = data.draft_order[data.draft_order.length - 1];
    expect(last.overall_pick).toBe(data.draft_order.length);
  });

  it("has compensatory picks only in rounds 3-7", () => {
    const data = generateTemplateDraftOrder(2026);
    const compPicks = data.draft_order.filter((e) => e.is_compensatory);

    expect(compPicks.length).toBeGreaterThan(0);
    expect(compPicks.every((e) => e.round >= 3)).toBe(true);
  });

  it("has sequential overall picks", () => {
    const data = generateTemplateDraftOrder(2026);
    data.draft_order.forEach((entry, i) => {
      expect(entry.overall_pick).toBe(i + 1);
    });
  });

  it("has sequential pick_in_round per round", () => {
    const data = generateTemplateDraftOrder(2026);
    let currentRound = 0;
    let expectedPick = 0;

    for (const entry of data.draft_order) {
      if (entry.round !== currentRound) {
        currentRound = entry.round;
        expectedPick = 1;
      } else {
        expectedPick++;
      }
      expect(entry.pick_in_round).toBe(expectedPick);
    }
  });

  it("uses only valid team abbreviations", () => {
    const data = generateTemplateDraftOrder(2026);
    const valid = new Set(VALID_TEAM_ABBREVIATIONS);

    for (const entry of data.draft_order) {
      expect(valid.has(entry.team_abbreviation)).toBe(true);
      expect(valid.has(entry.original_team_abbreviation)).toBe(true);
    }
  });
});
