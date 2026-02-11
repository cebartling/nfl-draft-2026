import { z } from 'zod';
import { UUIDSchema } from './common';

// RankingSource schema and type — matches backend RankingSourceResponse
export const RankingSourceSchema = z.object({
	id: UUIDSchema,
	name: z.string(),
	url: z.string().url().nullable().optional(),
	description: z.string().nullable().optional(),
});
export type RankingSource = z.infer<typeof RankingSourceSchema>;

// PlayerRanking schema and type — matches backend PlayerRankingResponse
export const PlayerRankingSchema = z.object({
	source_name: z.string(),
	source_id: UUIDSchema,
	rank: z.number(),
	scraped_at: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
});
export type PlayerRanking = z.infer<typeof PlayerRankingSchema>;

// SourceRanking schema and type — matches backend SourceRankingResponse
export const SourceRankingSchema = z.object({
	player_id: UUIDSchema,
	rank: z.number(),
	scraped_at: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
});
export type SourceRanking = z.infer<typeof SourceRankingSchema>;

// AllRankingEntry schema and type — matches backend AllRankingEntry
export const AllRankingEntrySchema = z.object({
	player_id: UUIDSchema,
	source_name: z.string(),
	rank: z.number(),
	scraped_at: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
});
export type AllRankingEntry = z.infer<typeof AllRankingEntrySchema>;

// Per-player ranking badge info (used in UI components)
export interface RankingBadge {
	source_name: string;
	abbreviation: string;
	rank: number;
}
