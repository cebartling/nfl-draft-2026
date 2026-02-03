import { z } from 'zod';
import { UUIDSchema } from './common';

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

// Team schema and type
export const TeamSchema = z.object({
	id: UUIDSchema,
	name: z.string(),
	abbreviation: z.string(),
	city: z.string(),
	conference: ConferenceSchema,
	division: DivisionSchema,
	logo_url: z.string().optional(),
});
export type Team = z.infer<typeof TeamSchema>;

// TeamNeed schema and type
export const TeamNeedSchema = z.object({
	id: UUIDSchema,
	team_id: UUIDSchema,
	position: z.string(),
	priority: z.number().min(1).max(10),
	notes: z.string().optional(),
});
export type TeamNeed = z.infer<typeof TeamNeedSchema>;
