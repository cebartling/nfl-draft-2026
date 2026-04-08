import { writeJsonFile } from "../shared/json-writer.js";
import { scrapeBeast } from "../scrapers/the-beast/index.js";
import { BeastDataSchema } from "../types/the-beast.js";

export interface TheBeastOptions {
  pdf: string;
  password: string;
  output: string;
  year: number;
}

export async function runTheBeastCommand(options: TheBeastOptions): Promise<void> {
  const { pdf, password, output, year } = options;

  if (!password) {
    throw new Error(
      "Beast PDF password is required. Pass --password <pwd> or set THE_BEAST_PASSWORD env var.",
    );
  }

  console.error("The Beast 2026 PDF scraper");
  console.error(`PDF:    ${pdf}`);
  console.error(`Year:   ${year}`);
  console.error(`Output: ${output}`);

  console.error("\nExtracting and parsing PDF...");
  const data = scrapeBeast({ pdfPath: pdf, password, draftYear: year });

  // Validate against the Zod schema before writing.
  const parsed = BeastDataSchema.safeParse(data);
  if (!parsed.success) {
    console.error("\nERROR: Parsed payload failed schema validation.");
    console.error(parsed.error.toString());
    throw new Error("Beast payload failed schema validation");
  }

  // Per-position counts for sanity-checking.
  const counts = new Map<string, number>();
  for (const p of data.prospects) {
    counts.set(p.position, (counts.get(p.position) ?? 0) + 1);
  }
  console.error("\nProspect counts by position:");
  for (const [pos, n] of [...counts.entries()].sort()) {
    console.error(`  ${pos.padEnd(6)} ${n}`);
  }
  console.error(`  TOTAL  ${data.meta.total_prospects}`);

  writeJsonFile(output, data);
  console.error(`\nWrote Beast data to: ${output}`);
}
