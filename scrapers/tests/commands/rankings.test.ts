import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { existsSync, mkdirSync, readFileSync, rmSync } from "fs";
import { join } from "path";
import { runRankingsCommand } from "../../src/commands/rankings.js";
import { RankingDataSchema } from "../../src/types/rankings.js";

const TEST_DIR = join(import.meta.dirname, "../../.test-output");

beforeEach(() => {
  mkdirSync(TEST_DIR, { recursive: true });
});

afterEach(() => {
  rmSync(TEST_DIR, { recursive: true, force: true });
});

describe("runRankingsCommand", () => {
  it("generates template output with valid JSON", async () => {
    const outputPath = join(TEST_DIR, "rankings_test.json");

    await runRankingsCommand({
      year: 2026,
      output: outputPath,
      template: true,
    });

    expect(existsSync(outputPath)).toBe(true);

    const content = readFileSync(outputPath, "utf-8");
    const data = RankingDataSchema.parse(JSON.parse(content));

    expect(data.meta.draft_year).toBe(2026);
    expect(data.meta.source).toBe("template");
    expect(data.rankings.length).toBeGreaterThanOrEqual(180);
  });

  it("validates output against Zod schema", async () => {
    const outputPath = join(TEST_DIR, "rankings_zod.json");

    await runRankingsCommand({
      year: 2026,
      output: outputPath,
      template: true,
    });

    const content = readFileSync(outputPath, "utf-8");
    const result = RankingDataSchema.safeParse(JSON.parse(content));
    expect(result.success).toBe(true);
  });
});
