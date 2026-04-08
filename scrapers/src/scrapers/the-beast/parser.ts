/**
 * Parser for the layout-preserving text extraction of Dane Brugler's
 * "The Beast 2026" PDF (output of `pdftotext -layout -upw <pwd>`).
 *
 * The parser is a pure function over text so it can be unit-tested with
 * fixture files committed to the repo. The PDF itself is password-protected
 * and never lands in version control.
 *
 * Strategy:
 *   1. Split text into position sections (QUARTERBACKS, RUNNING BACKS, ...)
 *   2. Within each section, find profile blocks keyed by `QB1`, `QB2`, etc.
 *   3. For each profile, extract the header line, header table, and the
 *      BACKGROUND / STATISTICS / STRENGTHS / WEAKNESSES / SUMMARY blocks.
 *   4. Top 100 page contributes overall ranks merged onto prospects by name.
 */

import type {
  BeastData,
  BeastMeta,
  BeastProspect,
  CollegeStatRow,
  Measurables,
} from "../../types/the-beast.js";

/**
 * Section configuration for The Beast 2026.
 * - `heading`: section header line as it appears in the PDF
 * - `positionCode`: our internal/database position enum value
 * - `headingPrefix`: prefix used on per-prospect profile headings
 *   (e.g. "G1 Olaivavega Ioane Penn State" for guards, mapped to OG)
 *
 * KICKERS / PUNTERS / LONG SNAPPERS are table-only — they have no individual
 * profiles in the PDF, so we treat them as one logical "Specialists" section
 * and parse only their summary tables.
 */
export const POSITION_SECTIONS: Array<{
  heading: string;
  positionCode: string;
  headingPrefix: string;
  tableOnly?: boolean;
}> = [
  { heading: "QUARTERBACKS", positionCode: "QB", headingPrefix: "QB" },
  { heading: "RUNNING BACKS", positionCode: "RB", headingPrefix: "RB" },
  { heading: "WIDE RECEIVERS", positionCode: "WR", headingPrefix: "WR" },
  { heading: "TIGHT ENDS", positionCode: "TE", headingPrefix: "TE" },
  { heading: "OFFENSIVE TACKLES", positionCode: "OT", headingPrefix: "OT" },
  { heading: "GUARDS", positionCode: "OG", headingPrefix: "G" },
  { heading: "CENTERS", positionCode: "C", headingPrefix: "C" },
  { heading: "EDGE RUSHERS", positionCode: "DE", headingPrefix: "EDGE" },
  { heading: "DEFENSIVE TACKLES", positionCode: "DT", headingPrefix: "DT" },
  { heading: "LINEBACKERS", positionCode: "LB", headingPrefix: "LB" },
  { heading: "CORNERBACKS", positionCode: "CB", headingPrefix: "CB" },
  { heading: "SAFETIES", positionCode: "S", headingPrefix: "S" },
  { heading: "KICKERS", positionCode: "K", headingPrefix: "K", tableOnly: true },
  { heading: "PUNTERS", positionCode: "P", headingPrefix: "P", tableOnly: true },
];

const POSITION_CODE_PATTERN = "(?:QB|RB|WR|TE|OT|OG|G|C|EDGE|DE|DT|LB|CB|S|K|P)";

/**
 * Headings that terminate a position section without producing one of their
 * own. Without these, the trailing position section (Punters) runs to end-of-
 * document and the position-summary table parser scoops up Long Snapper rows
 * and the entire TOP 100 page as bogus punters. We don't ingest LS or Top 100
 * as separate sections — they exist only to bound the previous section.
 */
const BOUNDARY_HEADINGS: string[] = ["LONG SNAPPERS", "TOP 100"];

/**
 * Convert The Beast's 4-digit height encoding to total inches.
 * Format: `FIIE` = F feet, II inches (two digits), E eighths.
 *   `6046` -> 6'0 4/8" -> 72 + 0 + 0.5 = 72.5 -> rounds down to 72? we keep integer inches
 * To preserve precision but stay integer, we use rounding to nearest inch.
 *
 * Returns `null` for unparseable input.
 */
export function decodeHeight(raw: string | null | undefined): number | null {
  if (!raw) return null;
  const cleaned = raw.replace(/\D/g, "");
  if (cleaned.length !== 4) return null;
  const feet = parseInt(cleaned[0], 10);
  const inches = parseInt(cleaned.slice(1, 3), 10);
  const eighths = parseInt(cleaned[3], 10);
  if (Number.isNaN(feet) || Number.isNaN(inches) || Number.isNaN(eighths)) {
    return null;
  }
  // Round half-inch up so 6046 (6'0 4/8") becomes 73 — close enough for integer storage.
  const total = feet * 12 + inches + eighths / 8;
  return Math.round(total);
}

/** Parse a numeric measurement; returns null for "DNP", "-", or empty. */
export function parseNumeric(value: string | undefined): number | null {
  if (!value) return null;
  const trimmed = value.trim();
  if (!trimmed || trimmed === "-" || /^DNP$/i.test(trimmed)) return null;
  // Strip common units
  const cleaned = trimmed.replace(/lbs\.?$/i, "").trim();
  const num = parseFloat(cleaned);
  return Number.isFinite(num) ? num : null;
}

/** Parse fractional inches like `9 1/2` -> 9.5 or `9 5/8` -> 9.625. */
export function parseFractionalInches(value: string | undefined): number | null {
  if (!value) return null;
  const trimmed = value.trim();
  if (!trimmed || trimmed === "-" || /^DNP$/i.test(trimmed)) return null;
  const fractionMatch = trimmed.match(/^(\d+)\s+(\d+)\/(\d+)$/);
  if (fractionMatch) {
    const whole = parseInt(fractionMatch[1], 10);
    const num = parseInt(fractionMatch[2], 10);
    const den = parseInt(fractionMatch[3], 10);
    if (den !== 0) return whole + num / den;
  }
  const simpleFraction = trimmed.match(/^(\d+)\/(\d+)$/);
  if (simpleFraction) {
    return parseInt(simpleFraction[1], 10) / parseInt(simpleFraction[2], 10);
  }
  const num = parseFloat(trimmed);
  return Number.isFinite(num) ? num : null;
}

/**
 * Convert "Oct 01, 2003" / "October 1, 2003" -> "2003-10-01". Returns null on failure.
 */
export function parseBirthday(value: string | undefined): string | null {
  if (!value) return null;
  const trimmed = value.trim();
  if (!trimmed) return null;
  const date = new Date(trimmed);
  if (Number.isNaN(date.getTime())) return null;
  const yyyy = date.getUTCFullYear();
  const mm = String(date.getUTCMonth() + 1).padStart(2, "0");
  const dd = String(date.getUTCDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
}

/**
 * Split full text into position sections by header lines.
 * Returns a Map of position code -> section text.
 *
 * Section bounds also respect BOUNDARY_HEADINGS (e.g. "LONG SNAPPERS",
 * "TOP 100") so that the trailing position section doesn't bleed into
 * unrelated content at the end of the document.
 */
export function splitIntoSections(text: string): Map<string, string> {
  const sections = new Map<string, string>();
  const lines = text.split(/\r?\n/);

  // Find the line index of every header — both real position sections and
  // boundary headings. Boundary headings get an empty positionCode so they
  // are recognized by `splitIntoSections` as terminators only, not consumers.
  type Marker = { idx: number; positionCode: string };
  const markers: Marker[] = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();

    let matchedSection = false;
    for (const section of POSITION_SECTIONS) {
      if (line === section.heading) {
        markers.push({ idx: i, positionCode: section.positionCode });
        matchedSection = true;
        break;
      }
    }
    if (matchedSection) continue;

    for (const boundary of BOUNDARY_HEADINGS) {
      if (line === boundary) {
        markers.push({ idx: i, positionCode: "" });
        break;
      }
    }
  }

  for (let s = 0; s < markers.length; s++) {
    const start = markers[s];
    if (!start.positionCode) continue; // boundary-only marker; nothing to emit
    const end = markers[s + 1]?.idx ?? lines.length;
    sections.set(start.positionCode, lines.slice(start.idx + 1, end).join("\n"));
  }

  return sections;
}

/**
 * Find the per-prospect profile blocks within a section. Each profile begins
 * with a heading like `QB1 Fernando Mendoza Indiana` on its own line.
 *
 * @param sectionText - text of the position section (without the header line)
 * @param headingPrefix - the prefix used on profile headings in the PDF
 *                        (e.g. "QB", "EDGE", "G" for guards). Distinct from
 *                        the database position code.
 */
export function splitProfiles(
  sectionText: string,
  headingPrefix: string,
): Array<{ positionRank: number; headingLine: string; bodyText: string }> {
  const escapedPrefix = headingPrefix.replace(/[^A-Z]/g, "");
  // The heading must start the line and contain a position code + rank number.
  // We use multiline mode to detect headings.
  const headingRegex = new RegExp(`^${escapedPrefix}(\\d+)\\s+(.+)$`, "gm");
  const matches = [...sectionText.matchAll(headingRegex)];
  const profiles: Array<{ positionRank: number; headingLine: string; bodyText: string }> = [];

  for (let i = 0; i < matches.length; i++) {
    const match = matches[i];
    const startIdx = match.index ?? 0;
    // Truncate the body at the BEST OF THE REST table if it appears within this profile range.
    let endIdx = i + 1 < matches.length ? (matches[i + 1].index ?? sectionText.length) : sectionText.length;
    const slice = sectionText.slice(startIdx, endIdx);
    const botrIdx = slice.search(/\n\s*BEST OF THE REST\s*\n/);
    if (botrIdx >= 0) endIdx = startIdx + botrIdx;
    const headingLine = match[0].trim();
    const bodyText = sectionText.slice(startIdx, endIdx);
    const positionRank = parseInt(match[1], 10);
    profiles.push({ positionRank, headingLine, bodyText });
  }

  return profiles;
}

/**
 * Parse the per-prospect summary table at the top of a position section
 * (before the first individual profile). Each row has rank, name, school,
 * grade tier, year class, height, weight, 40 (10), hand, arm, wing, age.
 *
 * Used as a fallback to enrich profiles and (more importantly) as the only
 * source for table-only sections like KICKERS / PUNTERS.
 */
export function parsePositionSummaryTable(
  sectionText: string,
): Array<{
  rank: number;
  fullName: string;
  school: string;
  gradeTier: string | null;
  yearClass: string | null;
  height_raw: string | null;
  weight_pounds: number | null;
  forty_yard_dash: number | null;
  ten_yard_split: number | null;
  hand_size: number | null;
  arm_length: number | null;
  wingspan: number | null;
  age: number | null;
}> {
  const lines = sectionText.split(/\r?\n/);
  const rows: ReturnType<typeof parsePositionSummaryTable> = [];
  // Stop at the first per-prospect profile heading or the BEST OF THE REST table.
  for (const line of lines) {
    if (/^[A-Z]{1,4}\d+\s+[A-Z][a-z]/.test(line.trim())) break;
    if (/^\s*BEST OF THE REST\s*$/.test(line)) break;
    // A summary row begins with optional whitespace, an integer rank, then an ALL-CAPS name.
    const m = line.match(/^\s*(\d{1,3})\s+([A-Z][A-Z' .\-]+?)\s{2,}(.+)$/);
    if (!m) continue;
    const rank = parseInt(m[1], 10);
    const fullName = m[2].trim().replace(/\s+/g, " ");
    const rest = m[3].trim();
    // Split rest on 2+ spaces; columns are: SCHOOL GRADE [YEAR] HEIGHT WEIGHT 40-YD(10-YD) HAND ARM WING AGE
    // YEAR may be missing in KICKERS/PUNTERS tables.
    const parts = rest.split(/\s{2,}/).map((p) => p.trim()).filter(Boolean);
    if (parts.length < 5) continue;
    const school = parts[0];
    const gradeTier = parts[1] ?? null;
    // Heuristic: a token like "4JR"/"5SR" identifies a year class column.
    const hasYear = parts[2] && /^\d[A-Z]{2}$/.test(parts[2]);
    let cursor = hasYear ? 3 : 2;
    const yearClass = hasYear ? parts[2] : null;
    const height_raw = parts[cursor++] ?? null;
    const weight_pounds = parts[cursor] ? Math.round(parseNumeric(parts[cursor]) ?? NaN) : null;
    cursor++;
    const fortyCell = parts[cursor++] ?? "";
    // 40-YD column is e.g. "4.56 (1.57)" or "DNP (DNP)"
    const fortyMatch = fortyCell.match(/(\d\.\d{2}|DNP)\s*\(\s*(\d\.\d{2}|DNP)\s*\)/);
    const forty_yard_dash = fortyMatch ? parseNumeric(fortyMatch[1]) : null;
    const ten_yard_split = fortyMatch ? parseNumeric(fortyMatch[2]) : null;
    const hand_size = parseFractionalInches(parts[cursor++]);
    const arm_length = parseFractionalInches(parts[cursor++]);
    const wingspan = parseFractionalInches(parts[cursor++]);
    const ageRaw = parts[cursor++];
    const age = ageRaw ? parseNumeric(ageRaw) : null;

    rows.push({
      rank,
      fullName,
      school,
      gradeTier,
      yearClass,
      height_raw: /^\d{4}$/.test(height_raw ?? "") ? height_raw : null,
      weight_pounds: Number.isFinite(weight_pounds as number) ? weight_pounds : null,
      forty_yard_dash,
      ten_yard_split,
      hand_size,
      arm_length,
      wingspan,
      age,
    });
  }
  return rows;
}

/**
 * Parse the "BEST OF THE REST" table that appears after the individually-profiled
 * prospects in each position section. The table format is similar but slightly
 * different from the top-of-section table — no GRADE or YEAR columns, and adds BP
 * (bench press) and 3-CONE columns.
 *
 * Layout:
 *    NAME            SCHOOL    HEIGHT  WEIGHT  40 YD  20 YD  10 YD  VJ  BJ  SS  3-CONE  BP  HAND  ARM  WING
 *  31 NAME...
 */
export function parseBestOfTheRest(
  sectionText: string,
): Array<{
  rank: number;
  fullName: string;
  school: string;
  height_raw: string | null;
  weight_pounds: number | null;
  forty_yard_dash: number | null;
  twenty_yard_split: number | null;
  ten_yard_split: number | null;
  hand_size: number | null;
  arm_length: number | null;
  wingspan: number | null;
}> {
  const startRe = /\n\s*BEST OF THE REST\s*\n/;
  const startMatch = sectionText.match(startRe);
  if (!startMatch) return [];
  const startIdx = (startMatch.index ?? 0) + startMatch[0].length;
  // The BOTR table runs to the end of the section.
  const tableText = sectionText.slice(startIdx);
  const lines = tableText.split(/\r?\n/);
  const rows: ReturnType<typeof parseBestOfTheRest> = [];

  for (const line of lines) {
    const m = line.match(/^\s*(\d{1,3})\s+([A-Z][A-Z' .\-]+?)\s{2,}(.+)$/);
    if (!m) continue;
    const rank = parseInt(m[1], 10);
    const fullName = m[2].trim().replace(/\s+/g, " ");
    const rest = m[3].trim();
    const parts = rest.split(/\s{2,}/).map((p) => p.trim()).filter(Boolean);
    if (parts.length < 4) continue;
    const school = parts[0];
    const height_raw = /^\d{4}$/.test(parts[1] ?? "") ? parts[1] : null;
    const weight_pounds = Math.round(parseNumeric(parts[2]) ?? NaN);
    rows.push({
      rank,
      fullName,
      school,
      height_raw,
      weight_pounds: Number.isFinite(weight_pounds) ? weight_pounds : null,
      forty_yard_dash: parseNumeric(parts[3]),
      twenty_yard_split: parseNumeric(parts[4]),
      ten_yard_split: parseNumeric(parts[5]),
      // VJ at parts[6], BJ at 7, SS at 8, 3-CONE at 9, BP at 10
      hand_size: parseFractionalInches(parts[11]),
      arm_length: parseFractionalInches(parts[12]),
      wingspan: parseFractionalInches(parts[13]),
    });
  }

  return rows;
}

/**
 * Parse a profile heading like "QB1 Fernando Mendoza Indiana" or
 * "QB5 Cole Payton North Dakota State". The school is everything after
 * the name; we use the position-summary table to disambiguate, but as a
 * fallback we assume the first two tokens are first/last name and the
 * rest is the school. This handles ~95% of cases; multi-word last names
 * (rare) may need manual fixup.
 */
export function parseHeading(
  headingLine: string,
  positionCode: string,
): { firstName: string; lastName: string; school: string } | null {
  const re = new RegExp(`^${positionCode}\\d+\\s+(.+)$`);
  const m = headingLine.match(re);
  if (!m) return null;
  const tokens = m[1].trim().split(/\s+/);
  if (tokens.length < 3) return null;
  // Default: first token = first name, second = last name, rest = school.
  // Edge case for hyphenated/two-word last names is left to caller / fixup pass.
  const firstName = tokens[0];
  const lastName = tokens[1];
  const school = tokens.slice(2).join(" ");
  return { firstName, lastName, school };
}

/**
 * Pull the header table line out of the profile body. The header table looks like:
 *   GRADE   OVR. RANK   YEAR   BIRTHDAY      AGE     HT      WT       JERSEY
 *   1st round    3      4JR    Oct 01, 2003  22.56   6'5"   236 lbs.  No. 15
 *
 * We locate the labels line, then read the next non-empty line as the values.
 */
export function parseHeaderTable(bodyText: string): {
  gradeTier: string | null;
  overallRank: number | null;
  yearClass: string | null;
  birthday: string | null;
  age: number | null;
  height_inches: number | null;
  weight_pounds: number | null;
  jersey_number: string | null;
} {
  const result = {
    gradeTier: null as string | null,
    overallRank: null as number | null,
    yearClass: null as string | null,
    birthday: null as string | null,
    age: null as number | null,
    height_inches: null as number | null,
    weight_pounds: null as number | null,
    jersey_number: null as string | null,
  };

  const lines = bodyText.split(/\r?\n/);
  // Find the first line containing both "GRADE" and "JERSEY" labels.
  let labelIdx = -1;
  for (let i = 0; i < lines.length; i++) {
    if (/GRADE/.test(lines[i]) && /JERSEY/.test(lines[i])) {
      labelIdx = i;
      break;
    }
  }
  if (labelIdx === -1) return result;

  // Read the next non-empty line as values.
  let valuesLine = "";
  for (let i = labelIdx + 1; i < lines.length; i++) {
    if (lines[i].trim()) {
      valuesLine = lines[i];
      break;
    }
  }
  if (!valuesLine) return result;

  // Match patterns within the values line. We use regex anchors against well-known
  // shapes since the column layout is fixed but whitespace is variable.

  // Grade tier: "1st round", "2nd round", "4th-5th", "7th-FA", "FA"
  const gradeMatch = valuesLine.match(/(\d+(?:st|nd|rd|th)(?:[- ](?:round|FA|\d+(?:st|nd|rd|th)))?|FA)/);
  if (gradeMatch) result.gradeTier = gradeMatch[1].replace(/^(\d+(?:st|nd|rd|th)) round$/, "$1 round");

  // Year class: 4JR, 5SR, 6SR, 7SR, 4SR
  const yearMatch = valuesLine.match(/\b(\d[A-Z]{2})\b/);
  if (yearMatch) result.yearClass = yearMatch[1];

  // Birthday: e.g. "Oct 01, 2003"
  const birthdayMatch = valuesLine.match(/([A-Z][a-z]{2}\s+\d{1,2},\s*\d{4})/);
  if (birthdayMatch) result.birthday = parseBirthday(birthdayMatch[1]);

  // Age: a decimal number with two digits before the dot, between 19 and 30
  const ageMatch = valuesLine.match(/\b(2\d\.\d{2})\b/);
  if (ageMatch) result.age = parseFloat(ageMatch[1]);

  // Height like 6'5" or 6'2"
  const htMatch = valuesLine.match(/(\d)'(\d{1,2})"/);
  if (htMatch) {
    result.height_inches = parseInt(htMatch[1], 10) * 12 + parseInt(htMatch[2], 10);
  }

  // Weight like 236 lbs. or 211 lbs.
  const wtMatch = valuesLine.match(/(\d{2,3})\s*lbs\.?/);
  if (wtMatch) result.weight_pounds = parseInt(wtMatch[1], 10);

  // Jersey: "No. 15"
  const jerseyMatch = valuesLine.match(/No\.\s*(\d+[A-Za-z]?)/);
  if (jerseyMatch) result.jersey_number = jerseyMatch[1];

  // Overall rank: a small integer that appears between the grade tier and the year class.
  // We extract the standalone integer in the values line that's not the weight or jersey.
  const overallMatch = valuesLine.match(/round\s+(\d+)\s+\d[A-Z]{2}/);
  if (overallMatch) result.overallRank = parseInt(overallMatch[1], 10);

  return result;
}

/**
 * Pull the bullet list following a labeled section like STRENGTHS or WEAKNESSES.
 * The Beast uses `●` (U+25CF) as the bullet marker. The list ends at the next
 * uppercase section label (WEAKNESSES, SUMMARY, etc.) or the next prospect heading.
 */
export function parseBulletList(bodyText: string, sectionLabel: string): string[] {
  const escaped = sectionLabel.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  // Match the section label on its own line (allow leading/trailing whitespace).
  const sectionRe = new RegExp(`(?:^|\\n)\\s*${escaped}\\s*\\n([\\s\\S]*?)(?=\\n\\s*(?:STRENGTHS|WEAKNESSES|SUMMARY|STATISTICS AND MEASUREMENTS|BACKGROUND)\\s*\\n|\\n\\s*${POSITION_CODE_PATTERN}\\d+\\s|$)`, "i");
  const m = bodyText.match(sectionRe);
  if (!m) return [];
  const block = m[1];

  return block
    .split(/\r?\n/)
    .map((line) => line.replace(/^\s*[●•▪]/, "").trim())
    .filter((line) => line.length > 0 && !/^Back to table of contents/i.test(line) && !/^\d+$/.test(line));
}

/** Pull the prose block following a labeled section. */
export function parseProseBlock(bodyText: string, sectionLabel: string): string | null {
  const escaped = sectionLabel.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const sectionRe = new RegExp(`(?:^|\\n)\\s*${escaped}\\s*\\n([\\s\\S]*?)(?=\\n\\s*(?:STRENGTHS|WEAKNESSES|SUMMARY|STATISTICS AND MEASUREMENTS|BACKGROUND)\\s*\\n|\\n\\s*${POSITION_CODE_PATTERN}\\d+\\s|$)`, "i");
  const m = bodyText.match(sectionRe);
  if (!m) return null;
  return m[1]
    .split(/\r?\n/)
    .map((line) => line.replace(/^\s*Back to table of contents.*$/i, "").trim())
    .filter((line) => line.length > 0)
    .join(" ")
    .replace(/\s+/g, " ")
    .trim() || null;
}

/**
 * Extract NFL comparison from the SUMMARY prose. Brugler typically writes
 * "...reminiscent of <Player>" or "...version of <Player>" near the end.
 */
export function extractNflComparison(summary: string | null): string | null {
  if (!summary) return null;
  const patterns = [
    /reminiscent of ([A-Z][a-zA-Z'.\-]+(?:\s+[A-Z][a-zA-Z'.\-]+){1,2})/,
    /version of ([A-Z][a-zA-Z'.\-]+(?:\s+[A-Z][a-zA-Z'.\-]+){1,2})/,
    /compares (?:favorably )?to ([A-Z][a-zA-Z'.\-]+(?:\s+[A-Z][a-zA-Z'.\-]+){1,2})/,
    /like ([A-Z][a-zA-Z'.\-]+(?:\s+[A-Z][a-zA-Z'.\-]+){1,2})\s*(?:\.|$)/,
  ];
  for (const pat of patterns) {
    const m = summary.match(pat);
    if (m) return m[1].replace(/[.,]+$/, "").trim();
  }
  return null;
}

/**
 * Parse the year-by-year stats table. We treat each line that begins with a
 * year token (e.g. `2025: (16/16)`) as a stat row and pack remaining columns
 * into a generic shape. Position-specific column meaning is left to consumers.
 */
export function parseCollegeStats(bodyText: string): CollegeStatRow[] {
  const lines = bodyText.split(/\r?\n/);
  const startRe = /^\s*STATISTICS AND MEASUREMENTS\s*$/i;
  let inStats = false;
  const rows: CollegeStatRow[] = [];

  for (const line of lines) {
    if (startRe.test(line)) {
      inStats = true;
      continue;
    }
    if (!inStats) continue;
    if (/^\s*(STRENGTHS|WEAKNESSES|SUMMARY)\s*$/i.test(line)) break;

    const yearMatch = line.match(/^\s*(\d{4})\s*:?\s*(\([^)]*\))?\s*(.*)$/);
    if (yearMatch) {
      const year = yearMatch[1];
      const games = yearMatch[2] ?? "";
      const rest = (yearMatch[3] ?? "").trim();
      rows.push({
        year: games ? `${year}: ${games}` : year,
        notes: rest || null,
      });
    }
  }

  return rows;
}

/**
 * Parse the COMBINE / PRO DAY measurables row. Returns null if not found.
 *
 * Layout (after STATISTICS AND MEASUREMENTS):
 *      HT     WT    HAND   ARM    WING   40-YD 20-YD 10-YD VJ  BJ  SS  3C    NOTES
 * COMBINE 6046 236  9 1/2  31 7/8 76 3/4 DNP   DNP   DNP   DNP DNP DNP DNP   ...
 * PRO DAY DNP  236  DNP    DNP    DNP    DNP   DNP   DNP   DNP DNP DNP DNP   -
 */
/**
 * Tokenize a measurables row into 12 cells (HT, WT, HAND, ARM, WING, 40, 20,
 * 10, VJ, BJ, SS, 3C). The input row is the text after the label (COMBINE or
 * PRO DAY). Cells may be:
 *   - 4-digit height (HT only)
 *   - integer
 *   - decimal
 *   - fractional inches like "9 1/2" (3 whitespace-separated tokens)
 *   - "DNP" or "-"
 *
 * pdftotext -layout uses single spaces between most cells, and a fraction has
 * internal single spaces too, so a naive split won't work. We instead consume
 * tokens and merge fraction parts greedily by position-based shape: cells 2,3,4
 * (HAND, ARM, WING) can be fractional and may consume up to 3 tokens.
 */
export function tokenizeMeasurablesRow(rowText: string): string[] {
  const tokens = rowText.trim().split(/\s+/);
  const cells: string[] = [];
  let i = 0;

  // Helper: peek a token
  const peek = (offset = 0) => tokens[i + offset];

  // Cell 0: HT — 4-digit number, or "DNP", or "-"
  cells.push(tokens[i++] ?? "");
  // Cell 1: WT — integer or DNP
  cells.push(tokens[i++] ?? "");
  // Cells 2,3,4: HAND, ARM, WING — fractional inches or DNP
  for (let f = 0; f < 3; f++) {
    const t = tokens[i];
    if (t === undefined) {
      cells.push("");
      continue;
    }
    if (/^DNP$/i.test(t) || t === "-") {
      cells.push(t);
      i++;
      continue;
    }
    // Could be "9", "9 1/2", or "10" — peek next two for whole-fraction shape.
    if (peek(1) && /^\d+\/\d+$/.test(peek(1)!)) {
      cells.push(`${tokens[i]} ${tokens[i + 1]}`);
      i += 2;
    } else if (/^\d+\/\d+$/.test(t)) {
      cells.push(t);
      i++;
    } else {
      cells.push(t);
      i++;
    }
  }
  // Cells 5..11: 40, 20, 10, VJ, BJ, SS, 3C — single tokens (decimal or DNP)
  for (let s = 0; s < 7; s++) {
    cells.push(tokens[i++] ?? "");
  }
  return cells;
}

export function parseMeasurables(
  bodyText: string,
  label: "COMBINE" | "PRO DAY",
): Measurables | null {
  const escaped = label.replace(/ /g, "\\s+");
  const re = new RegExp(`^\\s*${escaped}\\s+(.+)$`, "m");
  const m = bodyText.match(re);
  if (!m) return null;

  const cells = tokenizeMeasurablesRow(m[1]);
  const get = (i: number) => cells[i];

  const measurables: Measurables = {
    height_raw: /^\d{4}$/.test(get(0) ?? "") ? get(0)! : null,
    weight_pounds: parseNumeric(get(1)) !== null ? Math.round(parseNumeric(get(1))!) : null,
    hand_size: parseFractionalInches(get(2)),
    arm_length: parseFractionalInches(get(3)),
    wingspan: parseFractionalInches(get(4)),
    forty_yard_dash: parseNumeric(get(5)),
    twenty_yard_split: parseNumeric(get(6)),
    ten_yard_split: parseNumeric(get(7)),
    vertical_jump: parseNumeric(get(8)),
    broad_jump: parseNumeric(get(9)) !== null ? Math.round(parseNumeric(get(9))!) : null,
    twenty_yard_shuttle: parseNumeric(get(10)),
    three_cone_drill: parseNumeric(get(11)),
    bench_press: null, // The Beast doesn't include BP in the per-prospect block; it's in the position summary table.
  };

  // If the row is entirely DNP, treat as null
  const allNull = Object.values(measurables).every((v) => v === null);
  return allNull ? null : measurables;
}

/** Parse a single profile body into a BeastProspect (mostly populated). */
export function parseProfile(
  positionCode: string,
  headingPrefix: string,
  positionRank: number,
  headingLine: string,
  bodyText: string,
): BeastProspect | null {
  const heading = parseHeading(headingLine, headingPrefix);
  if (!heading) return null;

  const header = parseHeaderTable(bodyText);
  const background = parseProseBlock(bodyText, "BACKGROUND");
  const summary = parseProseBlock(bodyText, "SUMMARY");
  const strengths = parseBulletList(bodyText, "STRENGTHS");
  const weaknesses = parseBulletList(bodyText, "WEAKNESSES");
  const collegeStats = parseCollegeStats(bodyText);
  const combine = parseMeasurables(bodyText, "COMBINE");
  const proDay = parseMeasurables(bodyText, "PRO DAY");
  const nflComparison = extractNflComparison(summary);

  // Prefer combine height_raw for the canonical height_raw, else null.
  const heightRaw = combine?.height_raw ?? proDay?.height_raw ?? null;

  // headingPrefix is consumed by parseHeading above; the variable is otherwise unused
  // but we keep it in the signature so callers pass the right PDF prefix.
  void headingPrefix;

  return {
    position: positionCode,
    position_rank: positionRank,
    overall_rank: header.overallRank,
    first_name: heading.firstName,
    last_name: heading.lastName,
    school: heading.school,
    grade_tier: header.gradeTier,
    year_class: header.yearClass,
    birthday: header.birthday,
    age: header.age,
    jersey_number: header.jersey_number,
    height_inches: header.height_inches,
    weight_pounds: header.weight_pounds,
    height_raw: heightRaw,
    forty_yard_dash: combine?.forty_yard_dash ?? proDay?.forty_yard_dash ?? null,
    ten_yard_split: combine?.ten_yard_split ?? proDay?.ten_yard_split ?? null,
    hand_size: combine?.hand_size ?? proDay?.hand_size ?? null,
    arm_length: combine?.arm_length ?? proDay?.arm_length ?? null,
    wingspan: combine?.wingspan ?? proDay?.wingspan ?? null,
    combine,
    pro_day: proDay,
    college_stats: collegeStats,
    background,
    strengths,
    weaknesses,
    summary,
    nfl_comparison: nflComparison,
  };
}

/**
 * Parse the entire layout-preserving text dump of The Beast 2026 PDF.
 * Returns a BeastData payload ready for JSON serialization.
 */
export function parseBeastText(text: string, draftYear: number, scrapedAt: string): BeastData {
  const sections = splitIntoSections(text);
  const prospects: BeastProspect[] = [];

  for (const section of POSITION_SECTIONS) {
    const sectionText = sections.get(section.positionCode);
    if (!sectionText) continue;

    // Track which ranks were already added by full profile parsing so we don't
    // duplicate them when scanning the BEST OF THE REST table.
    const profiledRanks = new Set<number>();

    if (!section.tableOnly) {
      const profiles = splitProfiles(sectionText, section.headingPrefix);
      for (const p of profiles) {
        const profile = parseProfile(
          section.positionCode,
          section.headingPrefix,
          p.positionRank,
          p.headingLine,
          p.bodyText,
        );
        if (profile) {
          prospects.push(profile);
          profiledRanks.add(profile.position_rank);
        }
      }

      // Best of the Rest: ranks beyond the profiled set, table-only.
      const botr = parseBestOfTheRest(sectionText);
      for (const row of botr) {
        if (profiledRanks.has(row.rank)) continue;
        const [first, ...rest] = row.fullName.split(/\s+/);
        const last = rest.join(" ") || "";
        prospects.push({
          position: section.positionCode,
          position_rank: row.rank,
          overall_rank: null,
          first_name: titleCase(first),
          last_name: titleCase(last),
          school: row.school,
          grade_tier: null,
          year_class: null,
          birthday: null,
          age: null,
          jersey_number: null,
          height_inches: decodeHeight(row.height_raw),
          weight_pounds: row.weight_pounds,
          height_raw: row.height_raw,
          forty_yard_dash: row.forty_yard_dash,
          ten_yard_split: row.ten_yard_split,
          hand_size: row.hand_size,
          arm_length: row.arm_length,
          wingspan: row.wingspan,
          combine: null,
          pro_day: null,
          college_stats: [],
          background: null,
          strengths: [],
          weaknesses: [],
          summary: null,
          nfl_comparison: null,
        });
      }
    } else {
      // Table-only sections (KICKERS, PUNTERS): parse the position summary table only.
      const summaryRows = parsePositionSummaryTable(sectionText);
      for (const row of summaryRows) {
        const [first, ...rest] = row.fullName.split(/\s+/);
        const last = rest.join(" ") || "";
        prospects.push({
          position: section.positionCode,
          position_rank: row.rank,
          overall_rank: null,
          first_name: titleCase(first),
          last_name: titleCase(last),
          school: row.school,
          grade_tier: row.gradeTier,
          year_class: row.yearClass,
          birthday: null,
          age: row.age,
          jersey_number: null,
          height_inches: decodeHeight(row.height_raw),
          weight_pounds: row.weight_pounds,
          height_raw: row.height_raw,
          forty_yard_dash: row.forty_yard_dash,
          ten_yard_split: row.ten_yard_split,
          hand_size: row.hand_size,
          arm_length: row.arm_length,
          wingspan: row.wingspan,
          combine: null,
          pro_day: null,
          college_stats: [],
          background: null,
          strengths: [],
          weaknesses: [],
          summary: null,
          nfl_comparison: null,
        });
      }
    }
  }

  const meta: BeastMeta = {
    version: "1.0.0",
    source: "the-beast-2026",
    source_url: "https://theathletic.com/thebeast",
    draft_year: draftYear,
    scraped_at: scrapedAt,
    total_prospects: prospects.length,
  };

  return { meta, prospects };
}

/** Title-case an ALL-CAPS name token, preserving simple punctuation. */
function titleCase(name: string): string {
  if (!name) return "";
  return name
    .toLowerCase()
    .split(/(\s+|-|')/)
    .map((part) => (part && /^[a-z]/.test(part) ? part[0].toUpperCase() + part.slice(1) : part))
    .join("");
}
