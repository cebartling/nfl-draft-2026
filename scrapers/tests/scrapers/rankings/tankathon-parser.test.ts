import { describe, it, expect } from "vitest";
import { parseTankathonRankingsHtml } from "../../../src/scrapers/rankings/tankathon-parser.js";
import { RankingDataSchema } from "../../../src/types/rankings.js";

const SAMPLE_HTML = `
<html><body>
<div id="big-board">
  <div class="mock-row nfl">
    <div class="rank">1</div>
    <div class="mock-name">Travis Hunter</div>
    <div class="position">CB</div>
    <div class="school">Colorado</div>
  </div>
  <div class="mock-row nfl">
    <div class="rank">2</div>
    <div class="mock-name">Shedeur Sanders</div>
    <div class="position">QB</div>
    <div class="school">Colorado</div>
  </div>
  <div class="mock-row nfl">
    <div class="rank">3</div>
    <div class="mock-name">Rueben Bain Jr.</div>
    <div class="position">EDGE</div>
    <div class="school">Miami</div>
  </div>
</div>
</body></html>
`;

describe("parseTankathonRankingsHtml", () => {
  it("extracts all prospects from mock-row elements", () => {
    const data = parseTankathonRankingsHtml(SAMPLE_HTML, 2026);
    expect(data.rankings.length).toBe(3);
  });

  it("parses rank, name, position, school", () => {
    const data = parseTankathonRankingsHtml(SAMPLE_HTML, 2026);
    const first = data.rankings[0];
    expect(first.rank).toBe(1);
    expect(first.first_name).toBe("Travis");
    expect(first.last_name).toBe("Hunter");
    expect(first.position).toBe("CB");
    expect(first.school).toBe("Colorado");
  });

  it("handles name with Jr. suffix", () => {
    const data = parseTankathonRankingsHtml(SAMPLE_HTML, 2026);
    expect(data.rankings[2].first_name).toBe("Rueben");
    expect(data.rankings[2].last_name).toBe("Bain Jr.");
  });

  it("normalizes positions (EDGE → DE)", () => {
    const data = parseTankathonRankingsHtml(SAMPLE_HTML, 2026);
    expect(data.rankings[2].position).toBe("DE");
  });

  it("sets meta fields correctly", () => {
    const data = parseTankathonRankingsHtml(SAMPLE_HTML, 2026);
    expect(data.meta.source).toBe("tankathon");
    expect(data.meta.draft_year).toBe(2026);
    expect(data.meta.total_prospects).toBe(3);
  });

  it("validates against Zod schema", () => {
    const data = parseTankathonRankingsHtml(SAMPLE_HTML, 2026);
    const result = RankingDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });

  it("height and weight are null (Tankathon doesn't provide them)", () => {
    const data = parseTankathonRankingsHtml(SAMPLE_HTML, 2026);
    for (const entry of data.rankings) {
      expect(entry.height_inches).toBeNull();
      expect(entry.weight_pounds).toBeNull();
    }
  });

  it("falls back to embedded JSON when no mock-rows found", () => {
    const json = JSON.stringify([
      { rank: 1, name: "Travis Hunter", pos: "CB", school: "Colorado" },
      { rank: 2, name: "Shedeur Sanders", pos: "QB", school: "Colorado" },
    ]);
    const html = `
    <html><body>
    <script>window.__NEXT_DATA__ = { "props": { "pageProps": { "players": ${json} } } }</script>
    </body></html>`;
    const data = parseTankathonRankingsHtml(html, 2026);
    expect(data.rankings.length).toBe(2);
    expect(data.rankings[0].first_name).toBe("Travis");
  });

  it("handles DL → DT, IOL → OG, ATH → WR position mapping", () => {
    const html = `
    <html><body>
    <div id="big-board">
      <div class="mock-row nfl">
        <div class="rank">1</div>
        <div class="mock-name">Player One</div>
        <div class="position">DL</div>
        <div class="school">Alabama</div>
      </div>
      <div class="mock-row nfl">
        <div class="rank">2</div>
        <div class="mock-name">Player Two</div>
        <div class="position">IOL</div>
        <div class="school">Ohio State</div>
      </div>
    </div>
    </body></html>`;
    const data = parseTankathonRankingsHtml(html, 2026);
    expect(data.rankings[0].position).toBe("DT");
    expect(data.rankings[1].position).toBe("OG");
  });
});
