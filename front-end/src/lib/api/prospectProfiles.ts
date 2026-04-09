import { z } from 'zod';
import { apiClient, ApiClientError } from './client';

/**
 * Zod schema mirroring the backend `ProspectProfileResponse` shape.
 * Backed by the `prospect_profiles` table; currently the only source is
 * Dane Brugler's "the-beast-2026" guide.
 */
export const ProspectProfileSchema = z.object({
	id: z.string().uuid(),
	player_id: z.string().uuid(),
	source: z.string(),
	grade_tier: z.string().nullable(),
	overall_rank: z.number().int().nullable(),
	position_rank: z.number().int(),
	year_class: z.string().nullable(),
	birthday: z.string().nullable(), // YYYY-MM-DD
	jersey_number: z.string().nullable(),
	height_raw: z.string().nullable(),
	nfl_comparison: z.string().nullable(),
	background: z.string().nullable(),
	summary: z.string().nullable(),
	strengths: z.array(z.string()),
	weaknesses: z.array(z.string()),
	college_stats: z.unknown().nullable(),
	scraped_at: z.string(), // YYYY-MM-DD
});

export type ProspectProfile = z.infer<typeof ProspectProfileSchema>;

/**
 * Lightweight summary returned by the bulk list endpoint. Drops the heavy
 * prose fields so the player list / prospects page can render grade tier
 * badges without paying the cost of background, summary, strengths, etc.
 */
export const ProspectProfileSummarySchema = z.object({
	player_id: z.string().uuid(),
	source: z.string(),
	grade_tier: z.string().nullable(),
	overall_rank: z.number().int().nullable(),
	position_rank: z.number().int(),
	nfl_comparison: z.string().nullable(),
});

export type ProspectProfileSummary = z.infer<typeof ProspectProfileSummarySchema>;

export const prospectProfilesApi = {
	/**
	 * Fetch the latest prospect profile for a player. Returns `null` when
	 * no profile exists (404), so callers can render conditionally.
	 */
	async getByPlayer(playerId: string): Promise<ProspectProfile | null> {
		try {
			return await apiClient.get(`/players/${playerId}/profile`, ProspectProfileSchema);
		} catch (e) {
			if (e instanceof ApiClientError && e.status === 404) {
				return null;
			}
			throw e;
		}
	},

	/**
	 * Fetch all profile summaries from a given source (default: the-beast-2026).
	 * Returns a Map keyed by player_id for O(1) lookup at render time.
	 */
	async loadSummariesBySource(
		source = 'the-beast-2026'
	): Promise<Map<string, ProspectProfileSummary>> {
		const list = await apiClient.get(
			`/prospect-profiles?source=${encodeURIComponent(source)}`,
			z.array(ProspectProfileSummarySchema)
		);
		const map = new Map<string, ProspectProfileSummary>();
		for (const entry of list) {
			map.set(entry.player_id, entry);
		}
		return map;
	},
};
