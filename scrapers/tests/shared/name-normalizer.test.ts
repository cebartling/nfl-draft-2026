import { describe, it, expect } from "vitest";
import { cleanName, normalizeLastName, nameKey } from "../../src/shared/name-normalizer.js";

describe("cleanName", () => {
  it("strips periods", () => {
    expect(cleanName("C.J.")).toBe("cj");
  });

  it("lowercases", () => {
    expect(cleanName("CAM")).toBe("cam");
  });

  it("collapses whitespace", () => {
    expect(cleanName("  De Von  ")).toBe("de von");
  });

  it("strips apostrophes for matching", () => {
    expect(cleanName("De\u2019Von")).toBe("devon");
    expect(cleanName("De\u2018Von")).toBe("devon");
    expect(cleanName("D'Angelo")).toBe("dangelo");
    expect(cleanName("O'Brien")).toBe("obrien");
  });

  it("strips hyphens for matching", () => {
    expect(cleanName("McNeil-Warren")).toBe("mcneilwarren");
    expect(cleanName("Jean-Baptiste")).toBe("jeanbaptiste");
  });
});

describe("normalizeLastName", () => {
  it("strips Jr suffix", () => {
    expect(normalizeLastName("Carmona Jr.")).toBe("carmona");
    expect(normalizeLastName("Carmona Jr")).toBe("carmona");
  });

  it("strips Sr suffix", () => {
    expect(normalizeLastName("Smith Sr")).toBe("smith");
  });

  it("strips Roman numeral suffixes", () => {
    expect(normalizeLastName("Lee III")).toBe("lee");
    expect(normalizeLastName("Johnson II")).toBe("johnson");
    expect(normalizeLastName("Williams IV")).toBe("williams");
  });

  it("handles names without suffixes", () => {
    expect(normalizeLastName("Ward")).toBe("ward");
    expect(normalizeLastName("Hunter")).toBe("hunter");
  });

  it("lowercases and strips periods and apostrophes", () => {
    expect(normalizeLastName("O'Brien")).toBe("obrien");
  });

  it("strips hyphens", () => {
    expect(normalizeLastName("McNeil-Warren")).toBe("mcneilwarren");
  });
});

describe("nameKey", () => {
  it("combines clean first and normalized last name", () => {
    expect(nameKey("C.J.", "Stroud")).toBe("cj stroud");
    expect(nameKey("Fernando", "Carmona Jr.")).toBe("fernando carmona");
    expect(nameKey("Will", "Lee III")).toBe("will lee");
    expect(nameKey("CAM", "WARD")).toBe("cam ward");
  });

  it("handles case insensitivity for matching", () => {
    expect(nameKey("cam", "ward")).toBe(nameKey("CAM", "WARD"));
  });

  it("handles period variations", () => {
    expect(nameKey("C.J.", "Stroud")).toBe(nameKey("CJ", "Stroud"));
  });

  it("handles suffix variations", () => {
    expect(nameKey("Fernando", "Carmona Jr.")).toBe(nameKey("Fernando", "Carmona"));
    expect(nameKey("Will", "Lee III")).toBe(nameKey("Will", "Lee"));
  });
});
