import { z } from 'zod';
import { UUIDSchema } from './common';

// DraftStatus schema and type
export const DraftStatusSchema = z.enum(['NotStarted', 'InProgress', 'Paused', 'Completed']);
export type DraftStatus = z.infer<typeof DraftStatusSchema>;

// Draft schema and type — matches backend DraftResponse
export const DraftSchema = z.object({
	id: UUIDSchema,
	name: z.string(),
	year: z.number(),
	status: DraftStatusSchema,
	rounds: z.number(),
	picks_per_round: z.number().nullable(),
	total_picks: z.number().nullable(),
	is_realistic: z.boolean().optional(),
});
export type Draft = z.infer<typeof DraftSchema>;

// DraftPick schema and type — matches backend DraftPickResponse
export const DraftPickSchema = z.object({
	id: UUIDSchema,
	draft_id: UUIDSchema,
	round: z.number(),
	pick_number: z.number(),
	overall_pick: z.number(),
	team_id: UUIDSchema,
	player_id: UUIDSchema.nullable().optional(),
	picked_at: z.string().nullable().optional(),
	original_team_id: UUIDSchema.nullable().optional(),
	is_compensatory: z.boolean(),
	is_traded: z.boolean(),
	notes: z.string().nullable().optional(),
});
export type DraftPick = z.infer<typeof DraftPickSchema>;

// SessionStatus schema and type
export const SessionStatusSchema = z.enum(['NotStarted', 'InProgress', 'Paused', 'Completed']);
export type SessionStatus = z.infer<typeof SessionStatusSchema>;

// ChartType schema and type
export const ChartTypeSchema = z.enum([
	'JimmyJohnson',
	'RichHill',
	'ChaseStudartAV',
	'FitzgeraldSpielberger',
	'PffWar',
	'SurplusValue',
]);
export type ChartType = z.infer<typeof ChartTypeSchema>;

// DraftSession schema and type — matches backend SessionResponse
export const DraftSessionSchema = z.object({
	id: UUIDSchema,
	draft_id: UUIDSchema,
	status: SessionStatusSchema,
	current_pick_number: z.number(),
	time_per_pick_seconds: z.number(),
	auto_pick_enabled: z.boolean(),
	chart_type: ChartTypeSchema,
	controlled_team_ids: z.array(UUIDSchema).default([]),
	started_at: z.string().nullable().optional(),
	completed_at: z.string().nullable().optional(),
});
export type DraftSession = z.infer<typeof DraftSessionSchema>;

// DraftEvent schema and type — matches backend EventResponse
export const DraftEventSchema = z.object({
	id: UUIDSchema,
	session_id: UUIDSchema,
	event_type: z.string(),
	event_data: z.record(z.string(), z.unknown()),
	created_at: z.string(),
});
export type DraftEvent = z.infer<typeof DraftEventSchema>;
