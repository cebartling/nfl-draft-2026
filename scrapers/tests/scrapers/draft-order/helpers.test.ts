import { describe, it, expect } from "vitest";
import {
  parseRoundNumber,
  extractAbbrFromSvgUrl,
} from "../../../src/scrapers/draft-order/helpers.js";

describe("parseRoundNumber", () => {
  it("parses ordinal round titles", () => {
    expect(parseRoundNumber("1st Round")).toBe(1);
    expect(parseRoundNumber("2nd Round")).toBe(2);
    expect(parseRoundNumber("3rd Round")).toBe(3);
    expect(parseRoundNumber("4th Round")).toBe(4);
    expect(parseRoundNumber("7th Round")).toBe(7);
  });

  it("handles whitespace", () => {
    expect(parseRoundNumber("  1st Round  ")).toBe(1);
  });

  it("returns null for non-numeric text", () => {
    expect(parseRoundNumber("No number here")).toBeNull();
    expect(parseRoundNumber("")).toBeNull();
  });
});

describe("extractAbbrFromSvgUrl", () => {
  it("extracts slug from relative URL", () => {
    expect(extractAbbrFromSvgUrl("/img/nfl/lv.svg")).toBe("lv");
    expect(extractAbbrFromSvgUrl("/img/nfl/nyj.svg")).toBe("nyj");
  });

  it("extracts slug from absolute URL", () => {
    expect(extractAbbrFromSvgUrl("https://www.tankathon.com/img/nfl/atl.svg")).toBe("atl");
  });

  it("returns null for non-svg URLs", () => {
    expect(extractAbbrFromSvgUrl("/img/nfl/logo.png")).toBeNull();
  });

  it("returns null for empty string", () => {
    expect(extractAbbrFromSvgUrl("")).toBeNull();
  });
});
