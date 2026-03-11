import { describe, it, expect } from "vitest";
import {
  normalizeSvgAbbreviation,
  resolveTeamAbbreviation,
  VALID_TEAM_ABBREVIATIONS,
} from "../../src/shared/team-abbreviations.js";

describe("normalizeSvgAbbreviation", () => {
  it("uppercases standard abbreviations", () => {
    expect(normalizeSvgAbbreviation("dal")).toBe("DAL");
    expect(normalizeSvgAbbreviation("nyg")).toBe("NYG");
  });

  it("maps wsh to WAS", () => {
    expect(normalizeSvgAbbreviation("wsh")).toBe("WAS");
  });

  it("maps jac to JAX", () => {
    expect(normalizeSvgAbbreviation("jac")).toBe("JAX");
  });

  it("handles already-uppercase input", () => {
    expect(normalizeSvgAbbreviation("DAL")).toBe("DAL");
  });
});

describe("resolveTeamAbbreviation", () => {
  it("resolves full team names", () => {
    expect(resolveTeamAbbreviation("Dallas Cowboys")).toBe("DAL");
    expect(resolveTeamAbbreviation("Green Bay Packers")).toBe("GB");
  });

  it("resolves city-only names", () => {
    expect(resolveTeamAbbreviation("green bay")).toBe("GB");
    expect(resolveTeamAbbreviation("kansas city")).toBe("KC");
    expect(resolveTeamAbbreviation("san francisco")).toBe("SF");
    expect(resolveTeamAbbreviation("tampa bay")).toBe("TB");
    expect(resolveTeamAbbreviation("new england")).toBe("NE");
    expect(resolveTeamAbbreviation("new orleans")).toBe("NO");
  });

  it("resolves two-team city names", () => {
    expect(resolveTeamAbbreviation("new york giants")).toBe("NYG");
    expect(resolveTeamAbbreviation("ny giants")).toBe("NYG");
    expect(resolveTeamAbbreviation("new york jets")).toBe("NYJ");
    expect(resolveTeamAbbreviation("ny jets")).toBe("NYJ");
    expect(resolveTeamAbbreviation("los angeles chargers")).toBe("LAC");
    expect(resolveTeamAbbreviation("la chargers")).toBe("LAC");
    expect(resolveTeamAbbreviation("los angeles rams")).toBe("LAR");
    expect(resolveTeamAbbreviation("la rams")).toBe("LAR");
  });

  it("is case insensitive", () => {
    expect(resolveTeamAbbreviation("DALLAS COWBOYS")).toBe("DAL");
    expect(resolveTeamAbbreviation("dallas")).toBe("DAL");
  });

  it("strips parenthetical text", () => {
    expect(resolveTeamAbbreviation("Dallas (from GB)")).toBe("DAL");
    expect(resolveTeamAbbreviation("New York Giants (from CHI)")).toBe("NYG");
  });

  it("returns null for unknown teams", () => {
    expect(resolveTeamAbbreviation("Unknown Team")).toBeNull();
    expect(resolveTeamAbbreviation("")).toBeNull();
  });
});

describe("VALID_TEAM_ABBREVIATIONS", () => {
  it("contains all 32 NFL teams", () => {
    expect(VALID_TEAM_ABBREVIATIONS).toHaveLength(32);
  });

  it("includes key abbreviations", () => {
    expect(VALID_TEAM_ABBREVIATIONS).toContain("DAL");
    expect(VALID_TEAM_ABBREVIATIONS).toContain("NYG");
    expect(VALID_TEAM_ABBREVIATIONS).toContain("NYJ");
    expect(VALID_TEAM_ABBREVIATIONS).toContain("WAS");
    expect(VALID_TEAM_ABBREVIATIONS).toContain("JAX");
    expect(VALID_TEAM_ABBREVIATIONS).toContain("KC");
    expect(VALID_TEAM_ABBREVIATIONS).toContain("SF");
    expect(VALID_TEAM_ABBREVIATIONS).toContain("GB");
    expect(VALID_TEAM_ABBREVIATIONS).toContain("LAR");
    expect(VALID_TEAM_ABBREVIATIONS).toContain("LAC");
  });
});
