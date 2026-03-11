import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { existsSync, mkdirSync, readFileSync, rmSync } from "fs";
import { join } from "path";
import { runCombineCommand } from "../../src/commands/combine.js";
import { CombineDataSchema } from "../../src/types/combine.js";

const TEST_DIR = join(import.meta.dirname, "../../.test-output");

beforeEach(() => {
  mkdirSync(TEST_DIR, { recursive: true });
});

afterEach(() => {
  rmSync(TEST_DIR, { recursive: true, force: true });
});

describe("runCombineCommand", () => {
  it("generates template output with valid JSON", async () => {
    const outputPath = join(TEST_DIR, "combine_test.json");

    await runCombineCommand({
      year: 2026,
      output: outputPath,
      template: true,
    });

    expect(existsSync(outputPath)).toBe(true);

    const content = readFileSync(outputPath, "utf-8");
    const data = CombineDataSchema.parse(JSON.parse(content));

    expect(data.meta.draft_year ?? data.meta.year).toBe(2026);
    expect(data.meta.source).toBe("template");
    expect(data.combine_results.length).toBeGreaterThan(0);
  });

  it("validates output against Zod schema", async () => {
    const outputPath = join(TEST_DIR, "combine_zod.json");

    await runCombineCommand({
      year: 2026,
      output: outputPath,
      template: true,
    });

    const content = readFileSync(outputPath, "utf-8");
    const result = CombineDataSchema.safeParse(JSON.parse(content));
    expect(result.success).toBe(true);
  });
});
