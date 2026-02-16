import { z } from 'zod';
import { UUIDSchema } from './common';
import type { RankingBadge } from './ranking';

// Position schema and type
export const PositionSchema = z.enum([
	'QB',
	'RB',
	'WR',
	'TE',
	'OT',
	'OG',
	'C',
	'DE',
	'DT',
	'LB',
	'CB',
	'S',
	'K',
	'P',
]);
export type Position = z.infer<typeof PositionSchema>;

export const OFFENSE_POSITIONS: Position[] = ['QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C'];
export const DEFENSE_POSITIONS: Position[] = ['DE', 'DT', 'LB', 'CB', 'S'];
export const SPECIAL_POSITIONS: Position[] = ['K', 'P'];

export type PositionGroup = 'offense' | 'defense' | 'special_teams';

export const POSITION_GROUPS: Record<PositionGroup, Position[]> = {
	offense: OFFENSE_POSITIONS,
	defense: DEFENSE_POSITIONS,
	special_teams: SPECIAL_POSITIONS,
};

// Player schema and type — matches backend PlayerResponse
export const PlayerSchema = z.object({
	id: UUIDSchema,
	first_name: z.string(),
	last_name: z.string(),
	position: PositionSchema,
	college: z.string().optional(),
	height_inches: z.number().nullable().optional(),
	weight_pounds: z.number().nullable().optional(),
	draft_year: z.number(),
	draft_eligible: z.boolean(),
});
export type Player = z.infer<typeof PlayerSchema>;

// FitGrade schema and type
export const FitGradeSchema = z.enum(['A', 'B', 'C', 'D', 'F']);
export type FitGrade = z.infer<typeof FitGradeSchema>;

// ScoutingReport schema and type — matches backend ScoutingReportResponse
export const ScoutingReportSchema = z.object({
	id: UUIDSchema,
	player_id: UUIDSchema,
	team_id: UUIDSchema,
	grade: z.number(),
	notes: z.string().nullable().optional(),
	fit_grade: FitGradeSchema.nullable().optional(),
	injury_concern: z.boolean(),
	character_concern: z.boolean(),
});
export type ScoutingReport = z.infer<typeof ScoutingReportSchema>;

// RankingBadge schema (embedded in AvailablePlayer response)
export const RankingBadgeSchema = z.object({
	source_name: z.string(),
	abbreviation: z.string(),
	rank: z.number(),
});

// AvailablePlayer schema — matches backend AvailablePlayerResponse
export const AvailablePlayerSchema = z.object({
	id: UUIDSchema,
	first_name: z.string(),
	last_name: z.string(),
	position: PositionSchema,
	college: z.string().nullable().optional(),
	height_inches: z.number().nullable().optional(),
	weight_pounds: z.number().nullable().optional(),
	draft_year: z.number(),
	draft_eligible: z.boolean(),
	scouting_grade: z.number().nullable().optional(),
	fit_grade: FitGradeSchema.nullable().optional(),
	injury_concern: z.boolean().nullable().optional(),
	character_concern: z.boolean().nullable().optional(),
	rankings: z.array(RankingBadgeSchema),
});
export type AvailablePlayer = z.infer<typeof AvailablePlayerSchema>;

/** Convert a Player to an AvailablePlayer with optional ranking badges. */
export function toAvailablePlayer(
	player: Player,
	rankings: RankingBadge[] = [],
): AvailablePlayer {
	return {
		...player,
		college: player.college ?? null,
		scouting_grade: null,
		fit_grade: null,
		injury_concern: null,
		character_concern: null,
		rankings,
	};
}

// CombineResults schema and type — matches backend CombineResultsResponse
export const CombineResultsSchema = z.object({
	id: UUIDSchema,
	player_id: UUIDSchema,
	year: z.number(),
	source: z.string().optional(),
	forty_yard_dash: z.number().nullable().optional(),
	bench_press: z.number().nullable().optional(),
	vertical_jump: z.number().nullable().optional(),
	broad_jump: z.number().nullable().optional(),
	three_cone_drill: z.number().nullable().optional(),
	twenty_yard_shuttle: z.number().nullable().optional(),
	arm_length: z.number().nullable().optional(),
	hand_size: z.number().nullable().optional(),
	wingspan: z.number().nullable().optional(),
	ten_yard_split: z.number().nullable().optional(),
	twenty_yard_split: z.number().nullable().optional(),
});
export type CombineResults = z.infer<typeof CombineResultsSchema>;

// Individual measurement score within RAS
export const MeasurementScoreSchema = z.object({
	measurement: z.string(),
	raw_value: z.number(),
	percentile: z.number(),
	score: z.number(),
});
export type MeasurementScore = z.infer<typeof MeasurementScoreSchema>;

// RAS (Relative Athletic Score) — matches backend RasScoreResponse
export const RasScoreSchema = z.object({
	player_id: UUIDSchema,
	overall_score: z.number().nullable(),
	size_score: z.number().nullable(),
	speed_score: z.number().nullable(),
	strength_score: z.number().nullable(),
	explosion_score: z.number().nullable(),
	agility_score: z.number().nullable(),
	measurements_used: z.number(),
	measurements_total: z.number(),
	individual_scores: z.array(MeasurementScoreSchema),
	explanation: z.string().nullable().optional(),
});
export type RasScore = z.infer<typeof RasScoreSchema>;
