import { z } from 'zod';
import { UUIDSchema } from './common';

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

// CombineResults schema and type — matches backend CombineResultsResponse
export const CombineResultsSchema = z.object({
	id: UUIDSchema,
	player_id: UUIDSchema,
	year: z.number(),
	forty_yard_dash: z.number().optional(),
	bench_press: z.number().optional(),
	vertical_jump: z.number().optional(),
	broad_jump: z.number().optional(),
	three_cone_drill: z.number().optional(),
	twenty_yard_shuttle: z.number().optional(),
});
export type CombineResults = z.infer<typeof CombineResultsSchema>;
