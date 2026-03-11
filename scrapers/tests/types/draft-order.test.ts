import { describe, it, expect } from "vitest";
import type { DraftOrderEntry, DraftOrderData } from "../../src/types/draft-order.js";

describe("DraftOrderData", () => {
  it("round-trips through JSON serialization", () => {
    const data: DraftOrderData = {
      meta: {
        version: "1.0",
        last_updated: "2026-03-10",
        sources: ["tankathon"],
        draft_year: 2026,
        total_rounds: 7,
        total_picks: 1,
      },
      draft_order: [
        {
          round: 1,
          pick_in_round: 1,
          overall_pick: 1,
          team_abbreviation: "NYG",
          original_team_abbreviation: "NYG",
          is_compensatory: false,
          notes: null,
        },
      ],
    };

    const json = JSON.stringify(data);
    const deserialized: DraftOrderData = JSON.parse(json);
    expect(deserialized.meta.draft_year).toBe(2026);
    expect(deserialized.draft_order).toHaveLength(1);
    expect(deserialized.draft_order[0].team_abbreviation).toBe("NYG");
  });

  it("supports compensatory picks with notes", () => {
    const entry: DraftOrderEntry = {
      round: 3,
      pick_in_round: 33,
      overall_pick: 100,
      team_abbreviation: "DAL",
      original_team_abbreviation: "KC",
      is_compensatory: true,
      notes: "Compensatory selection",
    };

    expect(entry.is_compensatory).toBe(true);
    expect(entry.notes).toBe("Compensatory selection");
  });
});
