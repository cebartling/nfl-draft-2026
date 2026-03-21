import { describe, it, expect } from "vitest";
import { parseDraftTekHtml } from "../../../src/scrapers/rankings/drafttek-parser.js";
import { RankingDataSchema } from "../../../src/types/rankings.js";

// 2026 DraftTek table.player-info column layout:
//   0=Rank, 1=CNG(change), 2=Prospect, 3=College, 4=POS, 5=Ht, 6=Wt, 7=CLS, 8=BIO
// All table selectors (player-info, bpa, pointed) share this layout.
const SAMPLE_HTML = `
<html><body>
<table class="bpa">
  <tr>
    <td>1</td>
    <td>+2</td>
    <td>Travis Hunter</td>
    <td>Colorado</td>
    <td>CB</td>
    <td>6-1</td>
    <td>185</td>
    <td>Jr</td>
  </tr>
  <tr>
    <td>2</td>
    <td></td>
    <td>Shedeur Sanders</td>
    <td>Colorado</td>
    <td>QB</td>
    <td>6-2</td>
    <td>215</td>
    <td>Sr</td>
  </tr>
  <tr>
    <td>3</td>
    <td>-1</td>
    <td>Rueben Bain Jr.</td>
    <td>Miami</td>
    <td>DE</td>
    <td>6-3</td>
    <td>250</td>
    <td>Jr</td>
  </tr>
</table>
</body></html>
`;

describe("parseDraftTekHtml", () => {
  it("extracts all rows from the table", () => {
    const data = parseDraftTekHtml(SAMPLE_HTML, 2026);
    expect(data.rankings.length).toBe(3);
  });

  it("parses rank, name, school, position correctly", () => {
    const data = parseDraftTekHtml(SAMPLE_HTML, 2026);
    const first = data.rankings[0];
    expect(first.rank).toBe(1);
    expect(first.first_name).toBe("Travis");
    expect(first.last_name).toBe("Hunter");
    expect(first.school).toBe("Colorado");
    expect(first.position).toBe("CB");
  });

  it("parses height and weight", () => {
    const data = parseDraftTekHtml(SAMPLE_HTML, 2026);
    expect(data.rankings[0].height_inches).toBe(73); // 6-1
    expect(data.rankings[0].weight_pounds).toBe(185);
    expect(data.rankings[1].height_inches).toBe(74); // 6-2
    expect(data.rankings[1].weight_pounds).toBe(215);
  });

  it("normalizes positions (DE → DE)", () => {
    const data = parseDraftTekHtml(SAMPLE_HTML, 2026);
    expect(data.rankings[2].position).toBe("DE");
  });

  it("handles name with Jr. suffix", () => {
    const data = parseDraftTekHtml(SAMPLE_HTML, 2026);
    expect(data.rankings[2].first_name).toBe("Rueben");
    expect(data.rankings[2].last_name).toBe("Bain Jr.");
  });

  it("sets meta fields correctly", () => {
    const data = parseDraftTekHtml(SAMPLE_HTML, 2026);
    expect(data.meta.source).toBe("drafttek");
    expect(data.meta.draft_year).toBe(2026);
    expect(data.meta.total_prospects).toBe(3);
  });

  it("validates against Zod schema", () => {
    const data = parseDraftTekHtml(SAMPLE_HTML, 2026);
    const result = RankingDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });

  it("handles missing height/weight gracefully", () => {
    const html = `
    <html><body>
    <table class="bpa">
      <tr>
        <td>1</td>
        <td></td>
        <td>Test Player</td>
        <td>Alabama</td>
        <td>WR</td>
        <td>-</td>
        <td>-</td>
        <td>Jr</td>
      </tr>
    </table>
    </body></html>`;
    const data = parseDraftTekHtml(html, 2026);
    expect(data.rankings[0].height_inches).toBeNull();
    expect(data.rankings[0].weight_pounds).toBeNull();
  });

  it("skips header rows with non-position text", () => {
    const html = `
    <html><body>
    <table class="bpa">
      <tr>
        <td>Rank</td>
        <td>CNG</td>
        <td>Name</td>
        <td>College</td>
        <td>Position</td>
        <td>Height</td>
        <td>Weight</td>
        <td>Class</td>
      </tr>
      <tr>
        <td>1</td>
        <td>+1</td>
        <td>Travis Hunter</td>
        <td>Colorado</td>
        <td>CB</td>
        <td>6-1</td>
        <td>185</td>
        <td>Jr</td>
      </tr>
    </table>
    </body></html>`;
    const data = parseDraftTekHtml(html, 2026);
    expect(data.rankings.length).toBe(1);
  });

  it("handles alternate table selector (table.player-info)", () => {
    const html = `
    <html><body>
    <table class="player-info">
      <tr>
        <td>1</td>
        <td>+2</td>
        <td>Travis Hunter</td>
        <td>Colorado</td>
        <td>CB</td>
        <td>6-1</td>
        <td>185</td>
        <td>Jr</td>
      </tr>
    </table>
    </body></html>`;
    const data = parseDraftTekHtml(html, 2026);
    expect(data.rankings.length).toBe(1);
    expect(data.rankings[0].first_name).toBe("Travis");
  });

  it("handles alternate table selector (table tr.pointed)", () => {
    const html = `
    <html><body>
    <table>
      <tr class="pointed">
        <td>1</td>
        <td>+2</td>
        <td>Travis Hunter</td>
        <td>Colorado</td>
        <td>CB</td>
        <td>6-1</td>
        <td>185</td>
        <td>Jr</td>
      </tr>
    </table>
    </body></html>`;
    const data = parseDraftTekHtml(html, 2026);
    expect(data.rankings.length).toBe(1);
  });

  it("fixes missing space in initials (T.J.Hall → T.J. Hall)", () => {
    const html = `
    <html><body>
    <table class="bpa">
      <tr>
        <td>1</td>
        <td></td>
        <td>T.J.Hall</td>
        <td>Michigan</td>
        <td>DE</td>
        <td>6-4</td>
        <td>260</td>
        <td>Jr</td>
      </tr>
    </table>
    </body></html>`;
    const data = parseDraftTekHtml(html, 2026);
    expect(data.rankings[0].first_name).toBe("T.J.");
    expect(data.rankings[0].last_name).toBe("Hall");
  });
});
