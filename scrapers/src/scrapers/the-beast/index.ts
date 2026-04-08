/**
 * Driver for "The Beast 2026" PDF scraper. Decryption + text extraction is
 * delegated to `pdftotext` (Poppler), which is widely available and handles
 * AES-256 encrypted PDFs given the user password. The parser logic is in
 * `./parser.ts` so it remains pure and unit-testable.
 */

import { execFileSync } from "child_process";
import { existsSync } from "fs";

import { parseBeastText } from "./parser.js";
import type { BeastData } from "../../types/the-beast.js";

export interface ScrapeBeastOptions {
  pdfPath: string;
  password: string;
  draftYear: number;
  scrapedAt?: string; // override for tests; defaults to today
}

/** Returns the layout-preserving text extracted from the password-protected PDF. */
export function extractBeastText(pdfPath: string, password: string): string {
  if (!existsSync(pdfPath)) {
    throw new Error(`Beast PDF not found at: ${pdfPath}`);
  }
  // -layout preserves the column structure we rely on for parsing.
  // -enc UTF-8 ensures bullet characters (●) come through unchanged.
  // -upw passes the user password.
  const stdout = execFileSync(
    "pdftotext",
    ["-layout", "-enc", "UTF-8", "-upw", password, pdfPath, "-"],
    { encoding: "utf-8", maxBuffer: 256 * 1024 * 1024 },
  );
  return stdout;
}

/** Top-level entry point: extract + parse + return BeastData. */
export function scrapeBeast(options: ScrapeBeastOptions): BeastData {
  const text = extractBeastText(options.pdfPath, options.password);
  const scrapedAt = options.scrapedAt ?? new Date().toISOString().slice(0, 10);
  return parseBeastText(text, options.draftYear, scrapedAt);
}
