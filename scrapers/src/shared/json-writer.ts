import { existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { dirname } from "path";

const TEMPLATE_SOURCES = new Set(["template", "mock", "generated"]);

export function writeJsonFile(outputPath: string, data: unknown): void {
  mkdirSync(dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, JSON.stringify(data, null, 2) + "\n");
}

export function isTemplateData(data: Record<string, unknown>): boolean {
  const meta = data?.meta as Record<string, unknown> | undefined;
  if (!meta?.source) return false;
  return TEMPLATE_SOURCES.has(String(meta.source));
}

export function shouldPreventOverwrite(
  outputPath: string,
  newData: Record<string, unknown>,
): boolean {
  if (!existsSync(outputPath)) return false;
  if (!isTemplateData(newData)) return false;

  try {
    const existing = JSON.parse(readFileSync(outputPath, "utf-8"));
    return !isTemplateData(existing);
  } catch {
    return false;
  }
}
