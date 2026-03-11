import { describe, it, expect } from "vitest";
import type { RankingEntry, RankingData } from "../../src/types/rankings.js";

describe("RankingData", () => {
  it("round-trips through JSON serialization", () => {
    const data: RankingData = {
      meta: {
        version: "1.0",
        source: "tankathon",
        source_url: "https://tankathon.com/big_board",
        draft_year: 2026,
        scraped_at: "2026-03-10T00:00:00Z",
        total_prospects: 1,
      },
      rankings: [
        {
          rank: 1,
          first_name: "Cam",
          last_name: "Ward",
          position: "QB",
          school: "Miami",
          height_inches: 74,
          weight_pounds: 220,
        },
      ],
    };

    const json = JSON.stringify(data);
    const deserialized: RankingData = JSON.parse(json);
    expect(deserialized.meta.source).toBe("tankathon");
    expect(deserialized.rankings).toHaveLength(1);
    expect(deserialized.rankings[0].rank).toBe(1);
  });

  it("allows optional height and weight", () => {
    const entry: RankingEntry = {
      rank: 5,
      first_name: "Travis",
      last_name: "Hunter",
      position: "CB",
      school: "Colorado",
    };

    expect(entry.height_inches).toBeUndefined();
    expect(entry.weight_pounds).toBeUndefined();
  });
});
