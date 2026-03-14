import { describe, it, expect } from "vitest";
import { parseNflCombineResultsHtml } from "../../../src/scrapers/combine/nflcombineresults-parser.js";
import { CombineDataSchema } from "../../../src/types/combine.js";

// nflcombineresults.com uses a plain HTML table with standard column headers
const SAMPLE_HTML = `
<html><body>
<table class="datatable" id="datatable">
<thead>
    <tr>
        <th>Year</th>
        <th>Name</th>
        <th>College</th>
        <th>POS</th>
        <th>Height (in)</th>
        <th>Weight (lbs)</th>
        <th>Wonderlic</th>
        <th>40 Yard</th>
        <th>Bench Press</th>
        <th>Vert Leap (in)</th>
        <th>Broad Jump (in)</th>
        <th>Shuttle</th>
        <th>3Cone</th>
        <th>60Yd Shuttle</th>
    </tr>
</thead>
<tbody>
    <tr>
        <td>2026</td>
        <td>Cam Ward</td>
        <td>Miami (FL)</td>
        <td>QB</td>
        <td>74</td>
        <td>215</td>
        <td></td>
        <td>4.72</td>
        <td>18</td>
        <td>32.0</td>
        <td>108</td>
        <td>4.30</td>
        <td>7.05</td>
        <td></td>
    </tr>
    <tr>
        <td>2026</td>
        <td>Travis Hunter</td>
        <td>Colorado</td>
        <td>CB</td>
        <td>73</td>
        <td>188</td>
        <td></td>
        <td>4.38</td>
        <td></td>
        <td>40.5</td>
        <td>130</td>
        <td>4.05</td>
        <td></td>
        <td></td>
    </tr>
    <tr>
        <td>2026</td>
        <td>Shedeur Sanders</td>
        <td>Colorado</td>
        <td>QB</td>
        <td>73</td>
        <td>215</td>
        <td></td>
        <td>4.79</td>
        <td>15</td>
        <td>31.5</td>
        <td>105</td>
        <td>4.35</td>
        <td>7.12</td>
        <td></td>
    </tr>
    <tr>
        <td>2025</td>
        <td>Old Player</td>
        <td>Some School</td>
        <td>RB</td>
        <td>70</td>
        <td>200</td>
        <td></td>
        <td>4.50</td>
        <td>20</td>
        <td>35.0</td>
        <td>115</td>
        <td>4.20</td>
        <td>6.90</td>
        <td></td>
    </tr>
</tbody>
</table>
</body></html>
`;

describe("parseNflCombineResultsHtml", () => {
  it("extracts rows matching the specified year", () => {
    const data = parseNflCombineResultsHtml(SAMPLE_HTML, 2026);
    expect(data.combine_results.length).toBe(3);
  });

  it("filters out rows from other years", () => {
    const data = parseNflCombineResultsHtml(SAMPLE_HTML, 2026);
    const names = data.combine_results.map((e) => e.first_name);
    expect(names).not.toContain("Old");
  });

  it("parses player names correctly", () => {
    const data = parseNflCombineResultsHtml(SAMPLE_HTML, 2026);
    expect(data.combine_results[0].first_name).toBe("Cam");
    expect(data.combine_results[0].last_name).toBe("Ward");
    expect(data.combine_results[1].first_name).toBe("Travis");
    expect(data.combine_results[1].last_name).toBe("Hunter");
  });

  it("parses all measurables for complete entry", () => {
    const data = parseNflCombineResultsHtml(SAMPLE_HTML, 2026);
    const cam = data.combine_results[0];
    expect(cam.forty_yard_dash).toBe(4.72);
    expect(cam.bench_press).toBe(18);
    expect(cam.vertical_jump).toBe(32.0);
    expect(cam.broad_jump).toBe(108);
    expect(cam.three_cone_drill).toBe(7.05);
    expect(cam.twenty_yard_shuttle).toBe(4.30);
  });

  it("handles missing values as null", () => {
    const data = parseNflCombineResultsHtml(SAMPLE_HTML, 2026);
    const travis = data.combine_results[1];
    expect(travis.bench_press).toBeNull();
    expect(travis.three_cone_drill).toBeNull();
    // Present values
    expect(travis.forty_yard_dash).toBe(4.38);
    expect(travis.twenty_yard_shuttle).toBe(4.05);
  });

  it("normalizes positions", () => {
    const html = `
    <html><body>
    <table id="datatable">
    <thead><tr><th>Year</th><th>Name</th><th>College</th><th>POS</th><th>40 Yard</th></tr></thead>
    <tbody>
        <tr><td>2026</td><td>John Smith</td><td>State U</td><td>ILB</td><td>4.55</td></tr>
        <tr><td>2026</td><td>Bob Brown</td><td>Tech</td><td>FS</td><td>4.45</td></tr>
        <tr><td>2026</td><td>Mike Wilson</td><td>College</td><td>EDGE</td><td>4.60</td></tr>
    </tbody></table>
    </body></html>`;
    const data = parseNflCombineResultsHtml(html, 2026);
    expect(data.combine_results[0].position).toBe("LB");
    expect(data.combine_results[1].position).toBe("S");
    expect(data.combine_results[2].position).toBe("DE");
  });

  it("handles name suffixes (Jr., III)", () => {
    const html = `
    <html><body>
    <table id="datatable">
    <thead><tr><th>Year</th><th>Name</th><th>College</th><th>POS</th><th>40 Yard</th></tr></thead>
    <tbody>
        <tr><td>2026</td><td>Marvin Harrison Jr.</td><td>OSU</td><td>WR</td><td>4.40</td></tr>
        <tr><td>2026</td><td>Will Lee III</td><td>State</td><td>CB</td><td>4.35</td></tr>
    </tbody></table>
    </body></html>`;
    const data = parseNflCombineResultsHtml(html, 2026);
    expect(data.combine_results[0].first_name).toBe("Marvin");
    expect(data.combine_results[0].last_name).toBe("Harrison Jr.");
    expect(data.combine_results[1].first_name).toBe("Will");
    expect(data.combine_results[1].last_name).toBe("Lee III");
  });

  it("sets meta fields correctly", () => {
    const data = parseNflCombineResultsHtml(SAMPLE_HTML, 2026);
    expect(data.meta.source).toBe("nflcombineresults");
    expect(data.meta.year).toBe(2026);
    expect(data.meta.player_count).toBe(3);
    expect(data.meta.entry_count).toBe(3);
  });

  it("validates against Zod schema", () => {
    const data = parseNflCombineResultsHtml(SAMPLE_HTML, 2026);
    const result = CombineDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });

  it("throws when no table found", () => {
    const html = "<html><body>No table here</body></html>";
    expect(() => parseNflCombineResultsHtml(html, 2026)).toThrow();
  });

  it("returns empty results when no rows match year", () => {
    const html = `
    <html><body>
    <table id="datatable">
    <thead><tr><th>Year</th><th>Name</th><th>College</th><th>POS</th><th>40 Yard</th></tr></thead>
    <tbody>
        <tr><td>2025</td><td>Old Player</td><td>School</td><td>QB</td><td>4.50</td></tr>
    </tbody></table>
    </body></html>`;
    const data = parseNflCombineResultsHtml(html, 2026);
    expect(data.combine_results.length).toBe(0);
  });

  it("sets body measurement and split fields to null (not provided)", () => {
    const data = parseNflCombineResultsHtml(SAMPLE_HTML, 2026);
    const cam = data.combine_results[0];
    expect(cam.arm_length).toBeNull();
    expect(cam.hand_size).toBeNull();
    expect(cam.wingspan).toBeNull();
    expect(cam.ten_yard_split).toBeNull();
    expect(cam.twenty_yard_split).toBeNull();
  });

  it("handles rows without Year column by including all rows", () => {
    const html = `
    <html><body>
    <table id="datatable">
    <thead><tr><th>Name</th><th>College</th><th>POS</th><th>40 Yard</th></tr></thead>
    <tbody>
        <tr><td>Cam Ward</td><td>Miami</td><td>QB</td><td>4.72</td></tr>
        <tr><td>Travis Hunter</td><td>Colorado</td><td>CB</td><td>4.38</td></tr>
    </tbody></table>
    </body></html>`;
    const data = parseNflCombineResultsHtml(html, 2026);
    expect(data.combine_results.length).toBe(2);
  });
});
