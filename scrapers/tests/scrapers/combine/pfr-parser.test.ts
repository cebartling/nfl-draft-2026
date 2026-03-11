import { describe, it, expect } from "vitest";
import { parsePfrHtml } from "../../../src/scrapers/combine/pfr-parser.js";
import { CombineDataSchema } from "../../../src/types/combine.js";

const SAMPLE_HTML = `
<html><body>
<table id="combine" class="sortable stats_table">
<thead>
    <tr><th>Player</th><th>Pos</th></tr>
</thead>
<tbody>
    <tr>
        <th data-stat="player">Cam Ward</th>
        <td data-stat="pos">QB</td>
        <td data-stat="school_name">Miami (FL)</td>
        <td data-stat="forty_yd">4.72</td>
        <td data-stat="vertical">32.0</td>
        <td data-stat="bench_reps">18</td>
        <td data-stat="broad_jump">108</td>
        <td data-stat="cone">7.05</td>
        <td data-stat="shuttle">4.30</td>
        <td data-stat="arm_length">32.5</td>
        <td data-stat="hand_size">9.75</td>
        <td data-stat="wingspan">77.5</td>
        <td data-stat="ten_yd">1.65</td>
        <td data-stat="twenty_yd">2.72</td>
    </tr>
    <tr>
        <th data-stat="player">Travis Hunter</th>
        <td data-stat="pos">CB</td>
        <td data-stat="school_name">Colorado</td>
        <td data-stat="forty_yd">4.38</td>
        <td data-stat="vertical">40.5</td>
        <td data-stat="bench_reps">-</td>
        <td data-stat="broad_jump">130</td>
        <td data-stat="cone"></td>
        <td data-stat="shuttle">4.05</td>
        <td data-stat="arm_length"></td>
        <td data-stat="hand_size"></td>
        <td data-stat="wingspan"></td>
        <td data-stat="ten_yd">1.50</td>
        <td data-stat="twenty_yd"></td>
    </tr>
    <tr>
        <th data-stat="player">Shedeur Sanders</th>
        <td data-stat="pos">QB</td>
        <td data-stat="school_name">Colorado</td>
        <td data-stat="forty_yd">4.79</td>
        <td data-stat="vertical">31.5</td>
        <td data-stat="bench_reps">15</td>
        <td data-stat="broad_jump">105</td>
        <td data-stat="cone">7.12</td>
        <td data-stat="shuttle">4.35</td>
        <td data-stat="arm_length">33.0</td>
        <td data-stat="hand_size">9.50</td>
        <td data-stat="wingspan">78.0</td>
        <td data-stat="ten_yd">1.67</td>
        <td data-stat="twenty_yd">2.76</td>
    </tr>
</tbody>
</table>
</body></html>
`;

describe("parsePfrHtml", () => {
  it("extracts all rows from table#combine", () => {
    const data = parsePfrHtml(SAMPLE_HTML, 2026);
    expect(data.combine_results.length).toBe(3);
  });

  it("parses player names correctly", () => {
    const data = parsePfrHtml(SAMPLE_HTML, 2026);
    expect(data.combine_results[0].first_name).toBe("Cam");
    expect(data.combine_results[0].last_name).toBe("Ward");
    expect(data.combine_results[1].first_name).toBe("Travis");
    expect(data.combine_results[1].last_name).toBe("Hunter");
  });

  it("parses all measurables for complete entry", () => {
    const data = parsePfrHtml(SAMPLE_HTML, 2026);
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
    expect(cam.ten_yard_split).toBe(1.65);
    expect(cam.twenty_yard_split).toBe(2.72);
  });

  it("handles missing values (dash and empty)", () => {
    const data = parsePfrHtml(SAMPLE_HTML, 2026);
    const travis = data.combine_results[1];
    expect(travis.bench_press).toBeNull();
    expect(travis.three_cone_drill).toBeNull();
    expect(travis.arm_length).toBeNull();
    expect(travis.twenty_yard_split).toBeNull();
    // Present values
    expect(travis.forty_yard_dash).toBe(4.38);
    expect(travis.twenty_yard_shuttle).toBe(4.05);
  });

  it("normalizes positions", () => {
    const html = `
    <html><body>
    <table id="combine"><tbody>
        <tr>
            <th data-stat="player">John Smith</th>
            <td data-stat="pos">ILB</td>
            <td data-stat="forty_yd">4.55</td>
        </tr>
        <tr>
            <th data-stat="player">Bob Brown</th>
            <td data-stat="pos">FS</td>
            <td data-stat="forty_yd">4.45</td>
        </tr>
        <tr>
            <th data-stat="player">Mike Wilson</th>
            <td data-stat="pos">NT</td>
            <td data-stat="forty_yd">5.10</td>
        </tr>
    </tbody></table>
    </body></html>`;
    const data = parsePfrHtml(html, 2026);
    expect(data.combine_results[0].position).toBe("LB");
    expect(data.combine_results[1].position).toBe("S");
    expect(data.combine_results[2].position).toBe("DT");
  });

  it("handles name suffixes (Jr., III)", () => {
    const html = `
    <html><body>
    <table id="combine"><tbody>
        <tr>
            <th data-stat="player">Marvin Harrison Jr.</th>
            <td data-stat="pos">WR</td>
            <td data-stat="forty_yd">4.40</td>
        </tr>
        <tr>
            <th data-stat="player">Will Lee III</th>
            <td data-stat="pos">CB</td>
            <td data-stat="forty_yd">4.35</td>
        </tr>
    </tbody></table>
    </body></html>`;
    const data = parsePfrHtml(html, 2026);
    expect(data.combine_results[0].first_name).toBe("Marvin");
    expect(data.combine_results[0].last_name).toBe("Harrison Jr.");
    expect(data.combine_results[1].first_name).toBe("Will");
    expect(data.combine_results[1].last_name).toBe("Lee III");
  });

  it("sets meta fields correctly", () => {
    const data = parsePfrHtml(SAMPLE_HTML, 2026);
    expect(data.meta.source).toBe("pro_football_reference");
    expect(data.meta.year).toBe(2026);
    expect(data.meta.player_count).toBe(3);
    expect(data.meta.entry_count).toBe(3);
  });

  it("validates against Zod schema", () => {
    const data = parsePfrHtml(SAMPLE_HTML, 2026);
    const result = CombineDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });

  it("skips over_header and spacer rows", () => {
    const html = `
    <html><body>
    <table id="combine"><tbody>
        <tr class="over_header"><th>Section</th></tr>
        <tr>
            <th data-stat="player">Cam Ward</th>
            <td data-stat="pos">QB</td>
            <td data-stat="forty_yd">4.72</td>
        </tr>
        <tr class="spacer"><td colspan="20"></td></tr>
        <tr class="thead"><th>Player</th></tr>
        <tr>
            <th data-stat="player">Travis Hunter</th>
            <td data-stat="pos">CB</td>
            <td data-stat="forty_yd">4.38</td>
        </tr>
    </tbody></table>
    </body></html>`;
    const data = parsePfrHtml(html, 2026);
    expect(data.combine_results.length).toBe(2);
  });

  it("throws when no table#combine found", () => {
    const html = "<html><body>No table here</body></html>";
    expect(() => parsePfrHtml(html, 2026)).toThrow();
  });

  it("returns empty results for empty table", () => {
    const html = `
    <html><body>
    <table id="combine"><thead><tr><th>Player</th></tr></thead><tbody></tbody></table>
    </body></html>`;
    const data = parsePfrHtml(html, 2026);
    expect(data.combine_results.length).toBe(0);
    expect(data.meta.player_count).toBe(0);
  });
});
