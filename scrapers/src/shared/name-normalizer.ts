const SUFFIXES = new Set(["jr", "sr", "ii", "iii", "iv"]);

export function cleanName(name: string): string {
  return name
    .replace(/\./g, "")
    .replace(/[\u2019\u2018]/g, "'")
    .trim()
    .split(/\s+/)
    .join(" ")
    .toLowerCase();
}

export function normalizeLastName(name: string): string {
  const cleaned = name
    .replace(/\./g, "")
    .replace(/[\u2019\u2018]/g, "'");

  const parts = cleaned.trim().split(/\s+/);

  if (parts.length > 0) {
    const last = parts[parts.length - 1].toLowerCase();
    if (SUFFIXES.has(last)) {
      parts.pop();
    }
  }

  return parts.join(" ").toLowerCase();
}

export function nameKey(first: string, last: string): string {
  return `${cleanName(first)} ${normalizeLastName(last)}`;
}
