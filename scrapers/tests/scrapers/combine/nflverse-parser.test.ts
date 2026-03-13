import { describe, it, expect } from "vitest";
import { parseNflverseCsv } from "../../../src/scrapers/combine/nflverse-parser.js";
import { CombineDataSchema } from "../../../src/types/combine.js";

const SAMPLE_CSV = `season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,2025,NYG,1,1,WardCa00,,Cam Ward,QB,Miami (FL),74,215,4.72,18,32.0,108,7.05,4.30
2025,2025,CLE,1,2,HuntTr00,,Travis Hunter,CB,Colorado,73,205,4.38,,40.5,130,,4.05
2025,2025,TEN,1,3,SandSh00,,Shedeur Sanders,QB,Colorado,74,220,4.79,15,31.5,105,7.12,4.35
2024,2024,CHI,1,1,WillCa00,,Caleb Williams,QB,USC,73,214,4.59,12,33.0,115,7.01,4.20
`;

const CSV_WITH_MISSING = `season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,2025,NYG,1,1,,,John Smith,OLB,Alabama,75,245,,,,,,
`;

describe("parseNflverseCsv", () => {
  it("parses CSV rows into combine entries", () => {
    const data = parseNflverseCsv(SAMPLE_CSV, 2025);
    expect(data.combine_results.length).toBe(3);
  });

  it("filters by year (only rows matching requested season)", () => {
    const data = parseNflverseCsv(SAMPLE_CSV, 2025);
    expect(data.combine_results.length).toBe(3);
    // 2024 row excluded
    expect(data.combine_results.every((e) => e.year === 2025)).toBe(true);

    const data2024 = parseNflverseCsv(SAMPLE_CSV, 2024);
    expect(data2024.combine_results.length).toBe(1);
    expect(data2024.combine_results[0].first_name).toBe("Caleb");
  });

  it("splits player names correctly", () => {
    const data = parseNflverseCsv(SAMPLE_CSV, 2025);
    expect(data.combine_results[0].first_name).toBe("Cam");
    expect(data.combine_results[0].last_name).toBe("Ward");
    expect(data.combine_results[2].first_name).toBe("Shedeur");
    expect(data.combine_results[2].last_name).toBe("Sanders");
  });

  it("handles name suffixes (Jr., III)", () => {
    const csv = `season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,2025,ARI,1,5,,,Marvin Harrison Jr.,WR,Ohio State,73,205,4.40,,,,,
2025,2025,MIN,2,40,,,Will Lee III,CB,Alabama,71,190,4.35,,,,,
`;
    const data = parseNflverseCsv(csv, 2025);
    expect(data.combine_results[0].first_name).toBe("Marvin");
    expect(data.combine_results[0].last_name).toBe("Harrison Jr.");
    expect(data.combine_results[1].first_name).toBe("Will");
    expect(data.combine_results[1].last_name).toBe("Lee III");
  });

  it("handles hyphenated names", () => {
    const csv = `season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,2025,DEN,3,70,,,Quinshon Judkins-McNeil,RB,Ohio State,71,215,4.55,,,,,
`;
    const data = parseNflverseCsv(csv, 2025);
    expect(data.combine_results[0].first_name).toBe("Quinshon");
    expect(data.combine_results[0].last_name).toBe("Judkins-McNeil");
  });

  it("handles apostrophe in names", () => {
    const csv = `season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,2025,MIA,2,50,,,D'Angelo Smith,WR,Georgia,72,195,4.42,,,,,
`;
    const data = parseNflverseCsv(csv, 2025);
    expect(data.combine_results[0].first_name).toBe("D'Angelo");
    expect(data.combine_results[0].last_name).toBe("Smith");
  });

  it("normalizes positions", () => {
    const data = parseNflverseCsv(CSV_WITH_MISSING, 2025);
    expect(data.combine_results[0].position).toBe("LB"); // OLB → LB
  });

  it("maps measurement fields correctly", () => {
    const data = parseNflverseCsv(SAMPLE_CSV, 2025);
    const cam = data.combine_results[0];
    expect(cam.forty_yard_dash).toBe(4.72);
    expect(cam.bench_press).toBe(18);
    expect(cam.vertical_jump).toBe(32.0);
    expect(cam.broad_jump).toBe(108);
    expect(cam.three_cone_drill).toBe(7.05);
    expect(cam.twenty_yard_shuttle).toBe(4.30);
  });

  it("sets unmapped fields to null", () => {
    const data = parseNflverseCsv(SAMPLE_CSV, 2025);
    const cam = data.combine_results[0];
    expect(cam.arm_length).toBeNull();
    expect(cam.hand_size).toBeNull();
    expect(cam.wingspan).toBeNull();
    expect(cam.ten_yard_split).toBeNull();
    expect(cam.twenty_yard_split).toBeNull();
  });

  it("handles missing CSV values as null", () => {
    const data = parseNflverseCsv(SAMPLE_CSV, 2025);
    const travis = data.combine_results[1];
    expect(travis.bench_press).toBeNull();
    expect(travis.three_cone_drill).toBeNull();
    // Present values still parsed
    expect(travis.forty_yard_dash).toBe(4.38);
    expect(travis.vertical_jump).toBe(40.5);
  });

  it("handles all-null measurements", () => {
    const data = parseNflverseCsv(CSV_WITH_MISSING, 2025);
    const entry = data.combine_results[0];
    expect(entry.forty_yard_dash).toBeNull();
    expect(entry.bench_press).toBeNull();
    expect(entry.vertical_jump).toBeNull();
    expect(entry.broad_jump).toBeNull();
    expect(entry.three_cone_drill).toBeNull();
    expect(entry.twenty_yard_shuttle).toBeNull();
  });

  it("sets meta fields correctly", () => {
    const data = parseNflverseCsv(SAMPLE_CSV, 2025);
    expect(data.meta.source).toBe("nflverse");
    expect(data.meta.year).toBe(2025);
    expect(data.meta.player_count).toBe(3);
    expect(data.meta.entry_count).toBe(3);
    expect(data.meta.description).toContain("nflverse");
  });

  it("validates against Zod schema", () => {
    const data = parseNflverseCsv(SAMPLE_CSV, 2025);
    const result = CombineDataSchema.safeParse(data);
    expect(result.success).toBe(true);
  });

  it("returns empty results when no rows match year", () => {
    const data = parseNflverseCsv(SAMPLE_CSV, 2030);
    expect(data.combine_results.length).toBe(0);
    expect(data.meta.player_count).toBe(0);
  });

  it("handles empty CSV (header only)", () => {
    const csv = `season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
`;
    const data = parseNflverseCsv(csv, 2025);
    expect(data.combine_results.length).toBe(0);
  });

  it("rounds bench_press and broad_jump to integers", () => {
    const csv = `season,draft_year,draft_team,draft_round,draft_ovr,pfr_id,cfb_id,player_name,pos,school,ht,wt,forty,bench,vertical,broad_jump,cone,shuttle
2025,2025,NYG,1,1,,,Test Player,QB,Alabama,74,215,4.72,18.5,32.0,108.7,7.05,4.30
`;
    const data = parseNflverseCsv(csv, 2025);
    expect(data.combine_results[0].bench_press).toBe(19);
    expect(data.combine_results[0].broad_jump).toBe(109);
  });
});
