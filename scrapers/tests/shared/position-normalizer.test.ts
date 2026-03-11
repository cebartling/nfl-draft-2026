import { describe, it, expect } from "vitest";
import { normalizePosition } from "../../src/shared/position-normalizer.js";

describe("normalizePosition", () => {
  it("maps EDGE variants to DE", () => {
    expect(normalizePosition("DE")).toBe("DE");
    expect(normalizePosition("EDGE")).toBe("DE");
    expect(normalizePosition("EDGE/LB")).toBe("DE");
    expect(normalizePosition("LB/EDGE")).toBe("DE");
  });

  it("maps linebacker variants to LB", () => {
    expect(normalizePosition("LB")).toBe("LB");
    expect(normalizePosition("OLB")).toBe("LB");
    expect(normalizePosition("ILB")).toBe("LB");
    expect(normalizePosition("MLB")).toBe("LB");
  });

  it("maps interior line variants", () => {
    expect(normalizePosition("NT")).toBe("DT");
    expect(normalizePosition("DL")).toBe("DT");
    expect(normalizePosition("DT")).toBe("DT");
    expect(normalizePosition("C")).toBe("C");
    expect(normalizePosition("OG")).toBe("OG");
    expect(normalizePosition("G")).toBe("OG");
    expect(normalizePosition("IOL")).toBe("OG");
    expect(normalizePosition("OL")).toBe("OG");
  });

  it("maps safety variants to S", () => {
    expect(normalizePosition("FS")).toBe("S");
    expect(normalizePosition("SS")).toBe("S");
    expect(normalizePosition("DB")).toBe("S");
    expect(normalizePosition("SAF")).toBe("S");
  });

  it("maps backfield variants to RB", () => {
    expect(normalizePosition("FB")).toBe("RB");
    expect(normalizePosition("HB")).toBe("RB");
  });

  it("maps tackle variant T to OT", () => {
    expect(normalizePosition("T")).toBe("OT");
  });

  it("passes through canonical positions unchanged", () => {
    expect(normalizePosition("QB")).toBe("QB");
    expect(normalizePosition("WR")).toBe("WR");
    expect(normalizePosition("TE")).toBe("TE");
    expect(normalizePosition("OT")).toBe("OT");
    expect(normalizePosition("CB")).toBe("CB");
    expect(normalizePosition("RB")).toBe("RB");
    expect(normalizePosition("S")).toBe("S");
    expect(normalizePosition("LB")).toBe("LB");
    expect(normalizePosition("K")).toBe("K");
    expect(normalizePosition("P")).toBe("P");
  });

  it("is case insensitive", () => {
    expect(normalizePosition("de")).toBe("DE");
    expect(normalizePosition("qb")).toBe("QB");
    expect(normalizePosition("ilb")).toBe("LB");
  });

  it("trims whitespace", () => {
    expect(normalizePosition(" DE ")).toBe("DE");
    expect(normalizePosition("  QB  ")).toBe("QB");
  });

  it("returns uppercase passthrough for unknown positions", () => {
    expect(normalizePosition("ATH")).toBe("ATH");
    expect(normalizePosition("LS")).toBe("LS");
  });
});
