import { z } from 'zod';
import { UUIDSchema } from './common';
import { PositionSchema } from './player';

// Conference schema and type
export const ConferenceSchema = z.enum(['AFC', 'NFC']);
export type Conference = z.infer<typeof ConferenceSchema>;

// Division schema and type
export const DivisionSchema = z.enum([
	'AFC East',
	'AFC North',
	'AFC South',
	'AFC West',
	'NFC East',
	'NFC North',
	'NFC South',
	'NFC West',
]);
export type Division = z.infer<typeof DivisionSchema>;

// Team schema and type — matches backend TeamResponse
export const TeamSchema = z.object({
	id: UUIDSchema,
	name: z.string(),
	abbreviation: z.string(),
	city: z.string(),
	conference: ConferenceSchema,
	division: DivisionSchema,
});
export type Team = z.infer<typeof TeamSchema>;

// TeamNeed schema and type — matches backend TeamNeedResponse
export const TeamNeedSchema = z.object({
	id: UUIDSchema,
	team_id: UUIDSchema,
	position: PositionSchema,
	priority: z.number(),
});
export type TeamNeed = z.infer<typeof TeamNeedSchema>;

// PlayoffResult schema and type
export const PlayoffResultSchema = z.enum([
	'MissedPlayoffs',
	'WildCard',
	'Divisional',
	'Conference',
	'SuperBowlLoss',
	'SuperBowlWin',
]);
export type PlayoffResult = z.infer<typeof PlayoffResultSchema>;

// TeamSeason schema and type
export const TeamSeasonSchema = z.object({
	id: UUIDSchema,
	team_id: UUIDSchema,
	season_year: z.number(),
	wins: z.number(),
	losses: z.number(),
	ties: z.number(),
	playoff_result: PlayoffResultSchema.nullable().optional(),
	draft_position: z.number().nullable().optional(),
	win_percentage: z.number(),
});
export type TeamSeason = z.infer<typeof TeamSeasonSchema>;
