/**
 * Parse height string to total inches.
 * Handles formats: "6-3", "6'3", "6'3\"", "6-3\"", "75"
 */
export function parseHeight(height: string): number | null {
  const trimmed = height.trim();
  if (!trimmed || trimmed === "-") return null;

  // Try feet-inches formats: 6-3, 6'3, 6'3"
  const match = trimmed.match(/^(\d+)['\-](\d+)"?$/);
  if (match) {
    const feet = parseInt(match[1], 10);
    const inches = parseInt(match[2], 10);
    return feet * 12 + inches;
  }

  // Plain number (total inches)
  const num = parseInt(trimmed, 10);
  if (!isNaN(num) && String(num) === trimmed) {
    return num;
  }

  return null;
}

/**
 * Parse a rank number from formats like "1." or "1" or "01."
 */
export function parseRankNumber(text: string): number | null {
  const trimmed = text.trim().replace(/\.$/, "");
  if (!trimmed) return null;

  const num = parseInt(trimmed, 10);
  if (isNaN(num)) return null;

  return num;
}

const SUFFIXES = new Set(["jr.", "jr", "sr.", "sr", "ii", "iii", "iv"]);

/**
 * Split a full name into [first, last] parts.
 * Handles suffixes (Jr., Sr., II, III, IV) as part of last name.
 */
export function splitName(fullName: string): [string, string] {
  const parts = fullName.trim().split(/\s+/);
  if (parts.length === 0) return ["", ""];
  if (parts.length === 1) return [parts[0], ""];

  const first = parts[0];

  // Check if last part is a suffix — if so, last name is second-to-last + suffix
  if (parts.length >= 3 && SUFFIXES.has(parts[parts.length - 1].toLowerCase())) {
    const rest = parts.slice(1);
    return [first, rest.join(" ")];
  }

  return [first, parts.slice(1).join(" ")];
}
