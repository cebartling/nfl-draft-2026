import type { CombineData } from "../../types/combine.js";
import { parseNflComApi, type NflComCombineProfile } from "./nfl-com-parser.js";
import { launchBrowser, closeBrowser } from "../../shared/browser.js";

// Year is unused — the page URL is static; the API handles year filtering separately.
// Parameter kept for signature consistency with other combineUrl functions.
export function combineUrl(_year: number): string {
  return "https://www.nfl.com/combine/tracker/live-results/";
}

const API_BASE = "https://api.nfl.com/football/v2/combine/rankings";

/**
 * Rank attributes available on the NFL.com combine API.
 * Each returns a different subset of players (those who completed that drill).
 */
const RANK_ATTRIBUTES = [
  "FORTY_YARD_DASH",
  "VERTICAL_JUMP",
  "BROAD_JUMP",
  "BENCH_PRESS",
  "THREE_CONE_DRILL",
  "TWENTY_YARD_SHUTTLE",
];

/** API profile with id for deduplication. */
type ApiProfile = NflComCombineProfile & { id: string };

interface ApiResponse {
  combineProfiles: ApiProfile[];
  pagination: { limit: number; token: string | null };
}

/**
 * Obtain an auth token from NFL.com by loading the page and capturing the identity API response.
 */
async function getAuthToken(): Promise<string> {
  const browser = await launchBrowser();
  const page = await browser.newPage();

  const tokenPromise = new Promise<string>((resolve) => {
    page.on("response", async (resp) => {
      if (resp.url().includes("/identity/v3/token")) {
        try {
          const data = await resp.json();
          if (data.accessToken) {
            resolve(data.accessToken);
          }
        } catch {
          // ignore parse errors
        }
      }
    });
  });

  try {
    await page.goto("https://www.nfl.com/combine/tracker/live-results/", {
      waitUntil: "domcontentloaded",
      timeout: 60000,
    });

    // Race between token capture and timeout
    const timeout = new Promise<never>((_, reject) =>
      setTimeout(() => reject(new Error("Failed to obtain NFL.com auth token within 15s")), 15000),
    );
    return await Promise.race([tokenPromise, timeout]);
  } finally {
    await page.close();
  }
}

/**
 * Fetch combine profiles from the NFL.com API for a specific rank attribute.
 */
async function fetchRankings(
  token: string,
  year: number,
  rankAttribute: string,
): Promise<ApiProfile[]> {
  const url = `${API_BASE}?limit=500&rankAttribute=${rankAttribute}&sortOrder=ASC&year=${year}`;
  const response = await fetch(url, {
    headers: {
      Authorization: `Bearer ${token}`,
      "User-Agent":
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    },
    signal: AbortSignal.timeout(30000),
  });

  if (!response.ok) {
    throw new Error(`NFL.com API returned HTTP ${response.status} for ${rankAttribute}`);
  }

  const data: ApiResponse = await response.json();
  return data.combineProfiles || [];
}

/**
 * Scrape NFL.com combine data via internal API.
 * Uses Playwright only to obtain the auth token, then fetches all rank attributes
 * via standard HTTP to collect maximum player coverage.
 */
export async function scrapeNflCom(year: number): Promise<CombineData> {
  console.error("Obtaining NFL.com auth token via Playwright...");

  try {
    const token = await getAuthToken();
    console.error("Auth token obtained successfully");

    // Fetch all rank attributes to get maximum player coverage
    const allProfiles = new Map<string, ApiProfile>();

    for (const attr of RANK_ATTRIBUTES) {
      try {
        const profiles = await fetchRankings(token, year, attr);
        console.error(`  ${attr}: ${profiles.length} players`);

        for (const profile of profiles) {
          const existing = allProfiles.get(profile.id);
          if (existing) {
            // Merge: backfill null fields from this response
            mergeProfile(existing, profile);
          } else {
            allProfiles.set(profile.id, { ...profile });
          }
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        console.error(`  ${attr}: failed (${message})`);
      }
    }

    console.error(`  Total unique players: ${allProfiles.size}`);

    const profiles = Array.from(allProfiles.values());
    return parseNflComApi(profiles, year);
  } finally {
    await closeBrowser();
  }
}

/**
 * Merge fields from source into target, only backfilling null values.
 */
function mergeProfile(target: ApiProfile, source: ApiProfile): void {
  if (target.fortyYardDash == null && source.fortyYardDash != null)
    target.fortyYardDash = source.fortyYardDash;
  if (target.benchPress == null && source.benchPress != null) target.benchPress = source.benchPress;
  if (target.verticalJump == null && source.verticalJump != null)
    target.verticalJump = source.verticalJump;
  if (target.broadJump == null && source.broadJump != null) target.broadJump = source.broadJump;
  if (target.threeConeDrill == null && source.threeConeDrill != null)
    target.threeConeDrill = source.threeConeDrill;
  if (target.twentyYardShuttle == null && source.twentyYardShuttle != null)
    target.twentyYardShuttle = source.twentyYardShuttle;
  if (target.armLength == null && source.armLength != null) target.armLength = source.armLength;
  if (target.handSize == null && source.handSize != null) target.handSize = source.handSize;
  if (target.wingspan == null && source.wingspan != null) target.wingspan = source.wingspan;
  if (target.tenYardSplit == null && source.tenYardSplit != null)
    target.tenYardSplit = source.tenYardSplit;
  if (target.twentyYardSplit == null && source.twentyYardSplit != null)
    target.twentyYardSplit = source.twentyYardSplit;
}
