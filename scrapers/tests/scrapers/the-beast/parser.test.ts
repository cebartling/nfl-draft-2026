import { describe, it, expect } from "vitest";
import { readFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

import {
  decodeHeight,
  parseFractionalInches,
  parseNumeric,
  parseBirthday,
  splitIntoSections,
  splitProfiles,
  parseHeading,
  parseHeaderTable,
  parseBulletList,
  parseProseBlock,
  extractNflComparison,
  parseMeasurables,
  parseBeastText,
} from "../../../src/scrapers/the-beast/parser.js";
import { BeastDataSchema } from "../../../src/types/the-beast.js";

const __dirname = dirname(fileURLToPath(import.meta.url));
const FIXTURE_PATH = join(__dirname, "../../fixtures/the-beast/qb-sample.txt");
const FIXTURE = readFileSync(FIXTURE_PATH, "utf-8");

describe("decodeHeight", () => {
  it("decodes 6046 (6'4 6/8\") to 77 inches (rounded)", () => {
    // Per The Beast glossary: digits are feet, inches (2 digits), eighths.
    // 6046 = 6'4 6/8" = 76.75 -> rounds to 77
    expect(decodeHeight("6046")).toBe(77);
  });
  it("decodes 6011 (6'1 1/8\") to 73 inches", () => {
    expect(decodeHeight("6011")).toBe(73);
  });
  it("decodes 6024 (6'2 4/8\") to 75 inches per Brugler example", () => {
    expect(decodeHeight("6024")).toBe(75);
  });
  it("returns null for invalid input", () => {
    expect(decodeHeight("")).toBeNull();
    expect(decodeHeight("abc")).toBeNull();
    expect(decodeHeight(null)).toBeNull();
  });
});

describe("parseFractionalInches", () => {
  it("parses '9 1/2' as 9.5", () => {
    expect(parseFractionalInches("9 1/2")).toBe(9.5);
  });
  it("parses '9 5/8' as 9.625", () => {
    expect(parseFractionalInches("9 5/8")).toBe(9.625);
  });
  it("parses bare number", () => {
    expect(parseFractionalInches("10")).toBe(10);
  });
  it("returns null for DNP", () => {
    expect(parseFractionalInches("DNP")).toBeNull();
  });
});

describe("parseNumeric", () => {
  it("parses lbs values", () => {
    expect(parseNumeric("236 lbs.")).toBe(236);
  });
  it("returns null for DNP", () => {
    expect(parseNumeric("DNP")).toBeNull();
  });
});

describe("parseBirthday", () => {
  it("converts 'Oct 01, 2003' to ISO", () => {
    expect(parseBirthday("Oct 01, 2003")).toBe("2003-10-01");
  });
  it("converts 'Dec 21, 2002' to ISO", () => {
    expect(parseBirthday("Dec 21, 2002")).toBe("2002-12-21");
  });
});

describe("splitIntoSections", () => {
  it("finds the QUARTERBACKS section", () => {
    const sections = splitIntoSections(FIXTURE);
    expect(sections.has("QB")).toBe(true);
    expect(sections.get("QB")).toContain("QB1 Fernando Mendoza");
  });
  it("respects the next section header as a boundary", () => {
    const sections = splitIntoSections(FIXTURE);
    // The fixture has a trailing RUNNING BACKS line; QB section should not include it
    expect(sections.get("QB")).not.toMatch(/RUNNING BACKS/);
  });
});

describe("splitProfiles", () => {
  it("finds two QB profiles", () => {
    const sections = splitIntoSections(FIXTURE);
    const profiles = splitProfiles(sections.get("QB")!, "QB");
    expect(profiles.length).toBe(2);
    expect(profiles[0].positionRank).toBe(1);
    expect(profiles[1].positionRank).toBe(2);
    expect(profiles[0].headingLine).toMatch(/Fernando Mendoza/);
  });
});

describe("parseHeading", () => {
  it("parses single-school name", () => {
    const heading = parseHeading("QB1 Fernando Mendoza Indiana", "QB");
    expect(heading).toEqual({
      firstName: "Fernando",
      lastName: "Mendoza",
      school: "Indiana",
    });
  });
  it("parses multi-word school", () => {
    const heading = parseHeading("QB5 Cole Payton North Dakota State", "QB");
    expect(heading).toEqual({
      firstName: "Cole",
      lastName: "Payton",
      school: "North Dakota State",
    });
  });
});

describe("parseHeaderTable", () => {
  it("extracts grade tier, overall rank, year class, birthday, age, ht, wt, jersey", () => {
    const sections = splitIntoSections(FIXTURE);
    const profiles = splitProfiles(sections.get("QB")!, "QB");
    const header = parseHeaderTable(profiles[0].bodyText);
    expect(header.gradeTier).toBe("1st round");
    expect(header.overallRank).toBe(3);
    expect(header.yearClass).toBe("4JR");
    expect(header.birthday).toBe("2003-10-01");
    expect(header.age).toBe(22.56);
    expect(header.height_inches).toBe(77); // 6'5" = 77
    expect(header.weight_pounds).toBe(236);
    expect(header.jersey_number).toBe("15");
  });
});

describe("parseBulletList", () => {
  it("extracts strength bullets", () => {
    const sections = splitIntoSections(FIXTURE);
    const profiles = splitProfiles(sections.get("QB")!, "QB");
    const strengths = parseBulletList(profiles[0].bodyText, "STRENGTHS");
    expect(strengths.length).toBe(3);
    expect(strengths[0]).toBe("Placeholder strength one");
  });
  it("extracts weakness bullets", () => {
    const sections = splitIntoSections(FIXTURE);
    const profiles = splitProfiles(sections.get("QB")!, "QB");
    const weaknesses = parseBulletList(profiles[0].bodyText, "WEAKNESSES");
    expect(weaknesses.length).toBe(2);
    expect(weaknesses[1]).toBe("Placeholder weakness two");
  });
});

describe("parseProseBlock", () => {
  it("collapses multi-line BACKGROUND prose into one string", () => {
    const sections = splitIntoSections(FIXTURE);
    const profiles = splitProfiles(sections.get("QB")!, "QB");
    const bg = parseProseBlock(profiles[0].bodyText, "BACKGROUND");
    expect(bg).toContain("Placeholder background prose");
    expect(bg).toContain("multi-line prose blocks");
  });

  it("extracts SUMMARY prose", () => {
    const sections = splitIntoSections(FIXTURE);
    const profiles = splitProfiles(sections.get("QB")!, "QB");
    const summary = parseProseBlock(profiles[0].bodyText, "SUMMARY");
    expect(summary).toContain("mid-level NFL starter");
  });
});

describe("extractNflComparison", () => {
  it("extracts 'version of <Player>'", () => {
    expect(
      extractNflComparison("He projects as a mid-level NFL starter and a more mobile version of Bernie Kosar."),
    ).toBe("Bernie Kosar");
  });
  it("extracts 'reminiscent of <Player>'", () => {
    expect(
      extractNflComparison("Floor of a backup, reminiscent of Daniel Jones with lesser physical traits."),
    ).toBe("Daniel Jones");
  });
  it("returns null when no comparison", () => {
    expect(extractNflComparison("No comparison present here.")).toBeNull();
  });
});

describe("parseMeasurables", () => {
  it("parses COMBINE row with mostly DNP entries", () => {
    const sections = splitIntoSections(FIXTURE);
    const profiles = splitProfiles(sections.get("QB")!, "QB");
    const combine = parseMeasurables(profiles[0].bodyText, "COMBINE");
    expect(combine).not.toBeNull();
    expect(combine!.height_raw).toBe("6046");
    expect(combine!.weight_pounds).toBe(236);
    expect(combine!.hand_size).toBe(9.5);
    expect(combine!.arm_length).toBe(31.875);
    expect(combine!.wingspan).toBe(76.75);
  });
});

describe("parseBeastText (integration)", () => {
  it("produces a valid BeastData payload", () => {
    const data = parseBeastText(FIXTURE, 2026, "2026-04-08");
    const result = BeastDataSchema.safeParse(data);
    expect(result.success).toBe(true);
    expect(data.meta.total_prospects).toBe(2);
    expect(data.prospects[0].first_name).toBe("Fernando");
    expect(data.prospects[0].last_name).toBe("Mendoza");
    expect(data.prospects[0].overall_rank).toBe(3);
    expect(data.prospects[0].nfl_comparison).toBe("Bernie Kosar");
    expect(data.prospects[0].strengths.length).toBe(3);
    expect(data.prospects[0].weaknesses.length).toBe(2);
    expect(data.prospects[1].nfl_comparison).toBe("Daniel Jones");
  });
});

// Regression: the live PDF Punters section is followed by a "LONG SNAPPERS"
// section and then a "TOP 100" page, neither of which my POSITION_SECTIONS list
// contains. Without explicit boundary headings, the Punters section text runs
// to the end of the document and parsePositionSummaryTable scoops up Long
// Snapper rows and Top 100 rows as bogus punters. The bug surfaced as
// "BEAST: Ohio State" badges in the player UI for prospects whose grade_tier
// got set to a school name lifted from the Top 100 page.
const SPECIALISTS_FIXTURE = readFileSync(
  join(__dirname, "../../fixtures/the-beast/specialists-sample.txt"),
  "utf-8",
);

// Regression: pdftotext inserts a form-feed character (\f, U+000C) at PDF
// page boundaries, which lands as the first character of the heading line
// for the FIRST prospect on each new page. JS regex `^` in multiline mode
// does not reset on \f, so my heading regex `^EDGE\d+\s+...` failed to match
// `\fEDGE4 Keldric Faulk Auburn` and that prospect (and 50 others across the
// PDF) silently dropped out of the parser output.
const PAGE_BREAK_FIXTURE_RAW = readFileSync(
  join(__dirname, "../../fixtures/the-beast/page-break-sample.txt"),
  "utf-8",
);
// Inject the form-feed character before EDGE4 to simulate the pdftotext page
// break that the original PDF emits. Keeping the fixture file plain text.
const PAGE_BREAK_FIXTURE = PAGE_BREAK_FIXTURE_RAW.replace(
  "EDGE4 Keldric Faulk Auburn",
  "\fEDGE4 Keldric Faulk Auburn",
);

describe("parseBeastText page break handling", () => {
  it("parses prospects whose heading is preceded by a form-feed (page break)", () => {
    const data = parseBeastText(PAGE_BREAK_FIXTURE, 2026, "2026-04-08");
    const edge = data.prospects.filter((p) => p.position === "DE");
    const ranks = new Set(edge.map((p) => p.position_rank));
    expect(ranks.has(1)).toBe(true);
    expect(ranks.has(2)).toBe(true);
    expect(ranks.has(3)).toBe(true);
    // EDGE4 is the regression case — its heading line begins with a form feed.
    expect(ranks.has(4)).toBe(true);

    const faulk = edge.find((p) => p.position_rank === 4);
    expect(faulk).toBeDefined();
    expect(faulk!.first_name).toBe("Keldric");
    expect(faulk!.last_name).toBe("Faulk");
    expect(faulk!.school).toBe("Auburn");
    expect(faulk!.grade_tier).toBe("1st-2nd round");
    expect(faulk!.year_class).toBe("3JR");
  });

  it("does not let a form-feed-prefixed heading bleed into the previous prospect's body", () => {
    const data = parseBeastText(PAGE_BREAK_FIXTURE, 2026, "2026-04-08");
    const edge3 = data.prospects.find(
      (p) => p.position === "DE" && p.position_rank === 3,
    );
    expect(edge3).toBeDefined();
    // EDGE3's body should NOT contain Keldric Faulk's prose
    expect(edge3!.summary ?? "").not.toMatch(/Keldric/);
    expect(edge3!.background ?? "").not.toMatch(/Keldric/);
  });
});

describe("parseBeastText specialists section bounds", () => {
  it("does NOT spill Long Snappers or Top 100 rows into Punters", () => {
    const data = parseBeastText(SPECIALISTS_FIXTURE, 2026, "2026-04-08");
    const punters = data.prospects.filter((p) => p.position === "P");
    // Fixture defines exactly 3 punters; everything else (long snappers,
    // Top 100) must be excluded from the Punters bucket.
    expect(punters.length).toBe(3);
    expect(punters.map((p) => p.last_name)).toEqual(["Eckley", "Thorson", "Doman"]);
    // No punter should have a school that looks like a position-rank token
    // (e.g. "EDGE1", "RB1") — that's a smoking gun for Top 100 spillover.
    for (const p of punters) {
      expect(p.school).not.toMatch(/^(EDGE|QB|RB|WR|TE|OT|OG|G|C|DT|LB|CB|S|K|P|LS)\d+$/);
    }
    // No punter should have a grade_tier that looks like a school name
    // (the Top 100 row layout puts the school in the grade column position).
    for (const p of punters) {
      if (p.grade_tier) {
        expect(p.grade_tier).toMatch(/(\d+(?:st|nd|rd|th)|FA|^free)/i);
      }
    }
  });

  it("does not classify Top 100 prospects as Punters", () => {
    const data = parseBeastText(SPECIALISTS_FIXTURE, 2026, "2026-04-08");
    const punters = data.prospects.filter((p) => p.position === "P");
    const lastNames = new Set(punters.map((p) => p.last_name));
    // Arvell Reese (EDGE), Jeremiyah Love (RB), Fernando Mendoza (QB) appear
    // in the Top 100 page of the fixture. None should leak into Punters.
    expect(lastNames.has("Reese")).toBe(false);
    expect(lastNames.has("Love")).toBe(false);
    expect(lastNames.has("Mendoza")).toBe(false);
  });
});
