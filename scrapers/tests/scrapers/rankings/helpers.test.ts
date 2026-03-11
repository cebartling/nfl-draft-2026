import { describe, it, expect } from "vitest";
import { parseHeight, parseRankNumber, splitName } from "../../../src/scrapers/rankings/helpers.js";

describe("parseHeight", () => {
  it("parses dash format like '6-3' to 75 inches", () => {
    expect(parseHeight("6-3")).toBe(75);
  });

  it("parses apostrophe format like 6'3 to 75 inches", () => {
    expect(parseHeight("6'3")).toBe(75);
  });

  it("parses full format like 6'3\" to 75 inches", () => {
    expect(parseHeight('6\'3"')).toBe(75);
  });

  it("returns null for empty string", () => {
    expect(parseHeight("")).toBeNull();
  });

  it("returns null for dash placeholder", () => {
    expect(parseHeight("-")).toBeNull();
  });

  it("parses plain number as total inches", () => {
    expect(parseHeight("75")).toBe(75);
  });

  it("handles 6-0 edge case", () => {
    expect(parseHeight("6-0")).toBe(72);
  });
});

describe("parseRankNumber", () => {
  it("parses '1.' to 1", () => {
    expect(parseRankNumber("1.")).toBe(1);
  });

  it("parses '25' to 25", () => {
    expect(parseRankNumber("25")).toBe(25);
  });

  it("parses '01.' to 1", () => {
    expect(parseRankNumber("01.")).toBe(1);
  });

  it("returns null for non-numeric text", () => {
    expect(parseRankNumber("abc")).toBeNull();
  });

  it("returns null for empty string", () => {
    expect(parseRankNumber("")).toBeNull();
  });
});

describe("splitName", () => {
  it("splits simple two-part name", () => {
    expect(splitName("Travis Hunter")).toEqual(["Travis", "Hunter"]);
  });

  it("handles name with Jr. suffix", () => {
    expect(splitName("Rueben Bain Jr.")).toEqual(["Rueben", "Bain Jr."]);
  });

  it("handles name with III suffix", () => {
    expect(splitName("Will Lee III")).toEqual(["Will", "Lee III"]);
  });

  it("handles hyphenated last name", () => {
    expect(splitName("Dennis-Sutton Dani")).toEqual(["Dennis-Sutton", "Dani"]);
  });

  it("handles single name gracefully", () => {
    expect(splitName("Travis")).toEqual(["Travis", ""]);
  });

  it("trims whitespace", () => {
    expect(splitName("  Travis  Hunter  ")).toEqual(["Travis", "Hunter"]);
  });
});
