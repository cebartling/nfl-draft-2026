export function parseRoundNumber(text: string): number | null {
  const trimmed = text.trim().toLowerCase();
  const match = trimmed.match(/^(\d+)/);
  if (!match) return null;
  return parseInt(match[1], 10);
}

export function extractAbbrFromSvgUrl(src: string): string | null {
  const lastSegment = src.split("/").pop();
  if (!lastSegment) return null;
  if (!lastSegment.endsWith(".svg")) return null;
  return lastSegment.slice(0, -4) || null;
}
