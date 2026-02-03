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

// Player schema and type
export const PlayerSchema = z.object({
	id: UUIDSchema,
	first_name: z.string(),
	last_name: z.string(),
	position: PositionSchema,
	college: z.string().optional(),
	height_inches: z.number().optional(),
	weight_pounds: z.number().optional(),
	draft_year: z.number(),
	draft_eligible: z.boolean(),
	projected_round: z.number().optional(),
});
export type Player = z.infer<typeof PlayerSchema>;

// ScoutingReport schema and type
export const ScoutingReportSchema = z.object({
	id: UUIDSchema,
	player_id: UUIDSchema,
	team_id: UUIDSchema,
	grade: z.number().min(1).max(100),
	notes: z.string().optional(),
	strengths: z.string().optional(),
	weaknesses: z.string().optional(),
	created_at: z.string(),
	updated_at: z.string(),
});
export type ScoutingReport = z.infer<typeof ScoutingReportSchema>;

// CombineResults schema and type
export const CombineResultsSchema = z.object({
	id: UUIDSchema,
	player_id: UUIDSchema,
	forty_yard_dash: z.number().optional(),
	bench_press: z.number().optional(),
	vertical_jump: z.number().optional(),
	broad_jump: z.number().optional(),
	three_cone_drill: z.number().optional(),
	shuttle_run: z.number().optional(),
});
export type CombineResults = z.infer<typeof CombineResultsSchema>;
