import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { existsSync, mkdirSync, readFileSync, rmSync } from "fs";
import { join } from "path";
import { runDraftOrderCommand } from "../../src/commands/draft-order.js";
import { DraftOrderDataSchema } from "../../src/types/draft-order.js";

const TEST_DIR = join(import.meta.dirname, "../../.test-output");

beforeEach(() => {
  mkdirSync(TEST_DIR, { recursive: true });
});

afterEach(() => {
  rmSync(TEST_DIR, { recursive: true, force: true });
});

describe("runDraftOrderCommand", () => {
  it("generates template output with valid JSON", async () => {
    const outputPath = join(TEST_DIR, "draft_order_test.json");

    await runDraftOrderCommand({
      year: 2026,
      output: outputPath,
      template: true,
    });

    expect(existsSync(outputPath)).toBe(true);

    const content = readFileSync(outputPath, "utf-8");
    const data = DraftOrderDataSchema.parse(JSON.parse(content));

    expect(data.meta.draft_year).toBe(2026);
    expect(data.meta.source).toBe("template");
    expect(data.draft_order.length).toBeGreaterThan(224);
  });

  it("validates output against Zod schema", async () => {
    const outputPath = join(TEST_DIR, "draft_order_zod.json");

    await runDraftOrderCommand({
      year: 2026,
      output: outputPath,
      template: true,
    });

    const content = readFileSync(outputPath, "utf-8");
    const result = DraftOrderDataSchema.safeParse(JSON.parse(content));
    expect(result.success).toBe(true);
  });
});
