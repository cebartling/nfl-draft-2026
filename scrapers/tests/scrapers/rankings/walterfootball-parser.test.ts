import { describe, it, expect } from "vitest";
import { parseWalterFootballHtml } from "../../../src/scrapers/rankings/walterfootball-parser.js";
import { RankingDataSchema } from "../../../src/types/rankings.js";

// 2026 WalterFootball structure: alternating <b> pairs
//   <b>1.</b>  (rank-only tag matching /^\d+\.$/)
//   <b>Name, Position, School.</b>  (player info tag)
const SAMPLE_HTML = `
<html><body>
<div>
  <b>1.</b>
  <b>Travis Hunter, CB, Colorado.</b><br>
  Some scouting notes about this player.
  <br><br>
  <b>2.</b>
  <b>Shedeur Sanders, QB, Colorado.</b><br>
  More scouting notes.
  <br><br>
  <b>3.</b>
  <b>Rueben Bain Jr., DE, Miami.</b><br>
  Notes here.
</div>
</body></html>
`;

describe("parseWalterFootballHtml", () => {
  it("extracts all prospects", () => {
    const data = parseWalterFootballHtml(SAMPLE_HTML, 2026);
    expect(data.rankings.length).toBe(3);
  });

  it("parses rank, name, position, school", () => {
    const data = parseWalterFootballHtml(SAMPLE_HTML, 2026);
    const first = data.rankings[0];
    expect(first.rank).toBe(1);
    expect(first.first_name).toBe("Travis");
    expect(first.last_name).toBe("Hunter");
    expect(first.position).toBe("CB");
    expect(first.school).toBe("Colorado");
  });

  it("handles Jr. suffix in name", () => {
    const data = parseWalterFootballHtml(SAMPLE_HTML, 2026);
    expect(data.rankings[2].first_name).toBe("Rueben");
    expect(data.rankings[2].last_name).toBe("Bain Jr.");
  });

  it("normalizes positions", () => {
    const data = parseWalterFootballHtml(SAMPLE_HTML, 2026);
    expect(data.rankings[2].position).toBe("DE");
  });

  it("sets meta fields correctly", () => {
    const data = parseWalterFootballHtml(SAMPLE_HTML, 2026);
    expect(data.meta.source).toBe("walterfootball");
    expect(data.meta.draft_year).toBe(2026);
    expect(data.meta.total_prospects).toBe(3);
  });

  it("validates against Zod schema", () => {
    const data = parseWalterFootballHtml(SAMPLE_HTML, 2026);
    const result = RankingDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });

  it("height and weight are null (WalterFootball doesn't provide them)", () => {
    const data = parseWalterFootballHtml(SAMPLE_HTML, 2026);
    for (const entry of data.rankings) {
      expect(entry.height_inches).toBeNull();
      expect(entry.weight_pounds).toBeNull();
    }
  });

  it("handles linked names in <a> tags within the player tag", () => {
    const html = `
    <html><body>
    <div>
      <b>1.</b>
      <b><a href="/player">Travis Hunter</a>, CB, Colorado.</b><br>
    </div>
    </body></html>`;
    const data = parseWalterFootballHtml(html, 2026);
    expect(data.rankings.length).toBe(1);
    expect(data.rankings[0].first_name).toBe("Travis");
    expect(data.rankings[0].last_name).toBe("Hunter");
  });

  it("handles slash positions (takes first)", () => {
    const html = `
    <html><body>
    <div>
      <b>1.</b>
      <b>Francis Mauigoa, OT/G, Miami.</b><br>
    </div>
    </body></html>`;
    const data = parseWalterFootballHtml(html, 2026);
    expect(data.rankings[0].position).toBe("OT");
  });

  it("skips non-prospect bold text before the alternating pairs", () => {
    const html = `
    <html><body>
    <div>
      <b>NFL Draft Big Board 2026</b><br>
      <b>1.</b>
      <b>Travis Hunter, CB, Colorado.</b><br>
    </div>
    </body></html>`;
    const data = parseWalterFootballHtml(html, 2026);
    expect(data.rankings.length).toBe(1);
    expect(data.rankings[0].rank).toBe(1);
  });

  it("collapses extra whitespace in player tag", () => {
    const html = `
    <html><body>
    <div>
      <b>1.</b>
      <b>Jeremiyah   Love,   RB,   Notre Dame.</b><br>
    </div>
    </body></html>`;
    const data = parseWalterFootballHtml(html, 2026);
    expect(data.rankings.length).toBe(1);
    expect(data.rankings[0].first_name).toBe("Jeremiyah");
    expect(data.rankings[0].last_name).toBe("Love");
    expect(data.rankings[0].school).toBe("Notre Dame");
  });
});
