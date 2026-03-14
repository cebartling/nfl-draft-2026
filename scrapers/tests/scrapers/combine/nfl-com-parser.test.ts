import { describe, it, expect } from "vitest";
import {
  parseNflComHtml,
  parseNflComApi,
} from "../../../src/scrapers/combine/nfl-com-parser.js";
import { CombineDataSchema } from "../../../src/types/combine.js";

// --- HTML parser tests ---

const SAMPLE_HTML = `
<html><body>
<table class="nfl-o-table">
<thead>
    <tr>
        <th>Player</th>
        <th>POS</th>
        <th>School</th>
        <th>HT</th>
        <th>WT</th>
        <th>40YD</th>
        <th>BENCH</th>
        <th>VERT</th>
        <th>BROAD</th>
        <th>3CONE</th>
        <th>SHUTTLE</th>
        <th>ARM</th>
        <th>HAND</th>
        <th>WING</th>
    </tr>
</thead>
<tbody>
    <tr>
        <td>Cam Ward</td>
        <td>QB</td>
        <td>Miami (FL)</td>
        <td>6-2</td>
        <td>215</td>
        <td>4.72</td>
        <td>18</td>
        <td>32.0</td>
        <td>108</td>
        <td>7.05</td>
        <td>4.30</td>
        <td>32.5</td>
        <td>9.75</td>
        <td>77.5</td>
    </tr>
    <tr>
        <td>Travis Hunter</td>
        <td>CB</td>
        <td>Colorado</td>
        <td>6-1</td>
        <td>188</td>
        <td>4.38</td>
        <td>-</td>
        <td>40.5</td>
        <td>130</td>
        <td></td>
        <td>4.05</td>
        <td></td>
        <td></td>
        <td></td>
    </tr>
    <tr>
        <td>Shedeur Sanders</td>
        <td>QB</td>
        <td>Colorado</td>
        <td>6-1</td>
        <td>215</td>
        <td>4.79</td>
        <td>15</td>
        <td>31.5</td>
        <td>105</td>
        <td>7.12</td>
        <td>4.35</td>
        <td>33.0</td>
        <td>9.50</td>
        <td>78.0</td>
    </tr>
</tbody>
</table>
</body></html>
`;

describe("parseNflComHtml", () => {
  it("extracts all rows from table", () => {
    const data = parseNflComHtml(SAMPLE_HTML, 2026);
    expect(data.combine_results.length).toBe(3);
  });

  it("parses player names correctly", () => {
    const data = parseNflComHtml(SAMPLE_HTML, 2026);
    expect(data.combine_results[0].first_name).toBe("Cam");
    expect(data.combine_results[0].last_name).toBe("Ward");
    expect(data.combine_results[1].first_name).toBe("Travis");
    expect(data.combine_results[1].last_name).toBe("Hunter");
  });

  it("parses all measurables for complete entry", () => {
    const data = parseNflComHtml(SAMPLE_HTML, 2026);
    const cam = data.combine_results[0];
    expect(cam.forty_yard_dash).toBe(4.72);
    expect(cam.bench_press).toBe(18);
    expect(cam.vertical_jump).toBe(32.0);
    expect(cam.broad_jump).toBe(108);
    expect(cam.three_cone_drill).toBe(7.05);
    expect(cam.twenty_yard_shuttle).toBe(4.30);
    expect(cam.arm_length).toBe(32.5);
    expect(cam.hand_size).toBe(9.75);
    expect(cam.wingspan).toBe(77.5);
  });

  it("handles missing values (dash and empty)", () => {
    const data = parseNflComHtml(SAMPLE_HTML, 2026);
    const travis = data.combine_results[1];
    expect(travis.bench_press).toBeNull();
    expect(travis.three_cone_drill).toBeNull();
    expect(travis.arm_length).toBeNull();
    expect(travis.hand_size).toBeNull();
    expect(travis.wingspan).toBeNull();
    // Present values
    expect(travis.forty_yard_dash).toBe(4.38);
    expect(travis.twenty_yard_shuttle).toBe(4.05);
  });

  it("normalizes positions", () => {
    const html = `
    <html><body>
    <table class="nfl-o-table">
    <thead><tr><th>Player</th><th>POS</th><th>40YD</th></tr></thead>
    <tbody>
        <tr><td>John Smith</td><td>ILB</td><td>4.55</td></tr>
        <tr><td>Bob Brown</td><td>FS</td><td>4.45</td></tr>
        <tr><td>Mike Wilson</td><td>NT</td><td>5.10</td></tr>
        <tr><td>Jake Davis</td><td>EDGE</td><td>4.60</td></tr>
    </tbody></table>
    </body></html>`;
    const data = parseNflComHtml(html, 2026);
    expect(data.combine_results[0].position).toBe("LB");
    expect(data.combine_results[1].position).toBe("S");
    expect(data.combine_results[2].position).toBe("DT");
    expect(data.combine_results[3].position).toBe("DE");
  });

  it("handles name suffixes (Jr., III)", () => {
    const html = `
    <html><body>
    <table class="nfl-o-table">
    <thead><tr><th>Player</th><th>POS</th><th>40YD</th></tr></thead>
    <tbody>
        <tr><td>Marvin Harrison Jr.</td><td>WR</td><td>4.40</td></tr>
        <tr><td>Will Lee III</td><td>CB</td><td>4.35</td></tr>
    </tbody></table>
    </body></html>`;
    const data = parseNflComHtml(html, 2026);
    expect(data.combine_results[0].first_name).toBe("Marvin");
    expect(data.combine_results[0].last_name).toBe("Harrison Jr.");
    expect(data.combine_results[1].first_name).toBe("Will");
    expect(data.combine_results[1].last_name).toBe("Lee III");
  });

  it("sets meta fields correctly", () => {
    const data = parseNflComHtml(SAMPLE_HTML, 2026);
    expect(data.meta.source).toBe("nfl_com");
    expect(data.meta.year).toBe(2026);
    expect(data.meta.player_count).toBe(3);
    expect(data.meta.entry_count).toBe(3);
  });

  it("validates against Zod schema", () => {
    const data = parseNflComHtml(SAMPLE_HTML, 2026);
    const result = CombineDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });

  it("throws when no table found", () => {
    const html = "<html><body>No table here</body></html>";
    expect(() => parseNflComHtml(html, 2026)).toThrow();
  });

  it("returns empty results for empty table", () => {
    const html = `
    <html><body>
    <table class="nfl-o-table">
    <thead><tr><th>Player</th><th>POS</th></tr></thead>
    <tbody></tbody></table>
    </body></html>`;
    const data = parseNflComHtml(html, 2026);
    expect(data.combine_results.length).toBe(0);
    expect(data.meta.player_count).toBe(0);
  });

  it("handles table with data-* attributes on cells", () => {
    const html = `
    <html><body>
    <table class="nfl-o-table" data-table="combine">
    <thead><tr>
        <th data-col="player">Player</th>
        <th data-col="pos">POS</th>
        <th data-col="40yd">40YD</th>
        <th data-col="bench">BENCH</th>
    </tr></thead>
    <tbody>
        <tr>
            <td data-col="player">Test Player</td>
            <td data-col="pos">WR</td>
            <td data-col="40yd">4.35</td>
            <td data-col="bench">12</td>
        </tr>
    </tbody></table>
    </body></html>`;
    const data = parseNflComHtml(html, 2026);
    expect(data.combine_results.length).toBe(1);
    expect(data.combine_results[0].first_name).toBe("Test");
    expect(data.combine_results[0].last_name).toBe("Player");
    expect(data.combine_results[0].forty_yard_dash).toBe(4.35);
    expect(data.combine_results[0].bench_press).toBe(12);
  });

  it("sets ten_yard_split and twenty_yard_split to null (not provided by NFL.com HTML)", () => {
    const data = parseNflComHtml(SAMPLE_HTML, 2026);
    expect(data.combine_results[0].ten_yard_split).toBeNull();
    expect(data.combine_results[0].twenty_yard_split).toBeNull();
  });

  it("handles alternative table selectors", () => {
    const html = `
    <html><body>
    <div class="combine-tracker">
    <table>
    <thead><tr><th>Player</th><th>POS</th><th>40YD</th></tr></thead>
    <tbody>
        <tr><td>Cam Ward</td><td>QB</td><td>4.72</td></tr>
    </tbody></table>
    </div>
    </body></html>`;
    const data = parseNflComHtml(html, 2026);
    expect(data.combine_results.length).toBe(1);
  });
});

// --- API parser tests ---

const SAMPLE_PROFILES = [
  {
    person: { firstName: "Brenen", lastName: "Thompson", displayName: "Brenen Thompson" },
    fortyYardDash: { seconds: 4.26 },
    benchPress: null,
    verticalJump: null,
    broadJump: null,
    threeConeDrill: null,
    twentyYardShuttle: null,
    armLength: 29.375,
    handSize: 9,
    wingspan: null,
    tenYardSplit: { seconds: 1.54 },
    twentyYardSplit: null,
  },
  {
    person: { firstName: "Cam", lastName: "Ward", displayName: "Cam Ward" },
    fortyYardDash: { seconds: 4.72 },
    benchPress: 18,
    verticalJump: 32.0,
    broadJump: 108,
    threeConeDrill: 7.05,
    twentyYardShuttle: 4.3,
    armLength: 32.5,
    handSize: 9.75,
    wingspan: null,
    tenYardSplit: { seconds: 1.65 },
    twentyYardSplit: null,
  },
  {
    person: { firstName: "Travis", lastName: "Hunter", displayName: "Travis Hunter" },
    fortyYardDash: { seconds: 4.38 },
    benchPress: null,
    verticalJump: 40.5,
    broadJump: 130,
    threeConeDrill: null,
    twentyYardShuttle: 4.05,
    armLength: null,
    handSize: null,
    wingspan: null,
    tenYardSplit: { seconds: 1.5 },
    twentyYardSplit: null,
  },
];

describe("parseNflComApi", () => {
  it("extracts all profiles", () => {
    const data = parseNflComApi(SAMPLE_PROFILES, 2026);
    expect(data.combine_results.length).toBe(3);
  });

  it("parses player names from person object", () => {
    const data = parseNflComApi(SAMPLE_PROFILES, 2026);
    expect(data.combine_results[0].first_name).toBe("Brenen");
    expect(data.combine_results[0].last_name).toBe("Thompson");
    expect(data.combine_results[1].first_name).toBe("Cam");
    expect(data.combine_results[1].last_name).toBe("Ward");
  });

  it("parses nested fortyYardDash object correctly", () => {
    const data = parseNflComApi(SAMPLE_PROFILES, 2026);
    expect(data.combine_results[0].forty_yard_dash).toBe(4.26);
    expect(data.combine_results[1].forty_yard_dash).toBe(4.72);
  });

  it("parses nested tenYardSplit object correctly", () => {
    const data = parseNflComApi(SAMPLE_PROFILES, 2026);
    expect(data.combine_results[0].ten_yard_split).toBe(1.54);
    expect(data.combine_results[1].ten_yard_split).toBe(1.65);
  });

  it("parses flat numeric fields correctly", () => {
    const data = parseNflComApi(SAMPLE_PROFILES, 2026);
    const cam = data.combine_results[1];
    expect(cam.bench_press).toBe(18);
    expect(cam.vertical_jump).toBe(32.0);
    expect(cam.broad_jump).toBe(108);
    expect(cam.three_cone_drill).toBe(7.05);
    expect(cam.twenty_yard_shuttle).toBe(4.3);
    expect(cam.arm_length).toBe(32.5);
    expect(cam.hand_size).toBe(9.75);
  });

  it("handles null values correctly", () => {
    const data = parseNflComApi(SAMPLE_PROFILES, 2026);
    const brenen = data.combine_results[0];
    expect(brenen.bench_press).toBeNull();
    expect(brenen.vertical_jump).toBeNull();
    expect(brenen.broad_jump).toBeNull();
    expect(brenen.wingspan).toBeNull();
    expect(brenen.twenty_yard_split).toBeNull();
  });

  it("rounds bench_press and broad_jump to integers", () => {
    const profiles = [
      {
        person: { firstName: "Test", lastName: "Player", displayName: "Test Player" },
        fortyYardDash: null,
        benchPress: 18.7,
        verticalJump: null,
        broadJump: 108.3,
        threeConeDrill: null,
        twentyYardShuttle: null,
        armLength: null,
        handSize: null,
        wingspan: null,
        tenYardSplit: null,
        twentyYardSplit: null,
      },
    ];
    const data = parseNflComApi(profiles, 2026);
    expect(data.combine_results[0].bench_press).toBe(19);
    expect(data.combine_results[0].broad_jump).toBe(108);
  });

  it("sets meta fields correctly", () => {
    const data = parseNflComApi(SAMPLE_PROFILES, 2026);
    expect(data.meta.source).toBe("nfl_com");
    expect(data.meta.year).toBe(2026);
    expect(data.meta.player_count).toBe(3);
    expect(data.meta.entry_count).toBe(3);
  });

  it("validates against Zod schema", () => {
    const data = parseNflComApi(SAMPLE_PROFILES, 2026);
    const result = CombineDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });

  it("returns empty results for empty profiles array", () => {
    const data = parseNflComApi([], 2026);
    expect(data.combine_results.length).toBe(0);
    expect(data.meta.player_count).toBe(0);
  });

  it("normalizes position when provided in profile", () => {
    const profiles = [
      {
        person: { firstName: "Test", lastName: "Player", displayName: "Test Player" },
        position: "EDGE",
        fortyYardDash: { seconds: 4.5 },
        benchPress: null,
        verticalJump: null,
        broadJump: null,
        threeConeDrill: null,
        twentyYardShuttle: null,
        armLength: null,
        handSize: null,
        wingspan: null,
        tenYardSplit: null,
        twentyYardSplit: null,
      },
      {
        person: { firstName: "Another", lastName: "Player", displayName: "Another Player" },
        fortyYardDash: null,
        benchPress: null,
        verticalJump: null,
        broadJump: null,
        threeConeDrill: null,
        twentyYardShuttle: null,
        armLength: null,
        handSize: null,
        wingspan: null,
        tenYardSplit: null,
        twentyYardSplit: null,
      },
    ];
    const data = parseNflComApi(profiles, 2026);
    expect(data.combine_results[0].position).toBe("DE");
    expect(data.combine_results[1].position).toBe("");
  });
});
