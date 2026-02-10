import { z } from 'zod';
import { UUIDSchema } from './common';

// TradeStatus schema and type
export const TradeStatusSchema = z.enum(['Proposed', 'Accepted', 'Rejected']);
export type TradeStatus = z.infer<typeof TradeStatusSchema>;

// Trade schema and type — matches backend TradeResponse
export const TradeSchema = z.object({
	id: UUIDSchema,
	session_id: UUIDSchema,
	from_team_id: UUIDSchema,
	to_team_id: UUIDSchema,
	status: z.string(),
	from_team_value: z.number(),
	to_team_value: z.number(),
	value_difference: z.number(),
});
export type Trade = z.infer<typeof TradeSchema>;

// TradeProposal schema and type — matches backend TradeProposalResponse
export const TradeProposalSchema = z.object({
	trade: TradeSchema,
	from_team_picks: z.array(UUIDSchema),
	to_team_picks: z.array(UUIDSchema),
});
export type TradeProposal = z.infer<typeof TradeProposalSchema>;
