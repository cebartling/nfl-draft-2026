import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "fs";
import { join } from "path";
import { writeJsonFile, isTemplateData, shouldPreventOverwrite } from "../../src/shared/json-writer.js";

const TEST_DIR = join(import.meta.dirname, "../../.test-output");

beforeEach(() => {
  mkdirSync(TEST_DIR, { recursive: true });
});

afterEach(() => {
  rmSync(TEST_DIR, { recursive: true, force: true });
});

describe("writeJsonFile", () => {
  it("writes JSON with pretty formatting", () => {
    const outputPath = join(TEST_DIR, "test.json");
    const data = { meta: { source: "test" }, items: [1, 2, 3] };

    writeJsonFile(outputPath, data);

    const content = readFileSync(outputPath, "utf-8");
    expect(content).toContain('"source": "test"');
    expect(content.endsWith("\n")).toBe(true);
    const parsed = JSON.parse(content);
    expect(parsed).toEqual(data);
  });

  it("creates parent directories if needed", () => {
    const outputPath = join(TEST_DIR, "nested", "dir", "test.json");
    const data = { value: 42 };

    writeJsonFile(outputPath, data);

    expect(existsSync(outputPath)).toBe(true);
    const parsed = JSON.parse(readFileSync(outputPath, "utf-8"));
    expect(parsed.value).toBe(42);
  });
});

describe("isTemplateData", () => {
  it("identifies template data by source", () => {
    expect(isTemplateData({ meta: { source: "template" } })).toBe(true);
    expect(isTemplateData({ meta: { source: "mock" } })).toBe(true);
    expect(isTemplateData({ meta: { source: "generated" } })).toBe(true);
  });

  it("identifies real data", () => {
    expect(isTemplateData({ meta: { source: "pro_football_reference" } })).toBe(false);
    expect(isTemplateData({ meta: { source: "mockdraftable" } })).toBe(false);
    expect(isTemplateData({ meta: { source: "merged" } })).toBe(false);
    expect(isTemplateData({ meta: { source: "tankathon" } })).toBe(false);
  });

  it("handles missing meta gracefully", () => {
    expect(isTemplateData({})).toBe(false);
    expect(isTemplateData({ meta: {} })).toBe(false);
  });
});

describe("shouldPreventOverwrite", () => {
  it("prevents template data from overwriting real data", () => {
    const outputPath = join(TEST_DIR, "existing.json");
    writeFileSync(outputPath, JSON.stringify({ meta: { source: "merged" } }));

    const newData = { meta: { source: "template" } };
    expect(shouldPreventOverwrite(outputPath, newData)).toBe(true);
  });

  it("allows real data to overwrite template data", () => {
    const outputPath = join(TEST_DIR, "existing.json");
    writeFileSync(outputPath, JSON.stringify({ meta: { source: "template" } }));

    const newData = { meta: { source: "merged" } };
    expect(shouldPreventOverwrite(outputPath, newData)).toBe(false);
  });

  it("allows writing when no existing file", () => {
    const outputPath = join(TEST_DIR, "nonexistent.json");
    const newData = { meta: { source: "template" } };
    expect(shouldPreventOverwrite(outputPath, newData)).toBe(false);
  });

  it("allows real data to overwrite real data", () => {
    const outputPath = join(TEST_DIR, "existing.json");
    writeFileSync(outputPath, JSON.stringify({ meta: { source: "merged" } }));

    const newData = { meta: { source: "pro_football_reference" } };
    expect(shouldPreventOverwrite(outputPath, newData)).toBe(false);
  });
});
