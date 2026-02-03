import { z } from 'zod';
import { UUIDSchema } from './common';

// TradeStatus schema and type
export const TradeStatusSchema = z.enum(['Proposed', 'Accepted', 'Rejected', 'Expired']);
export type TradeStatus = z.infer<typeof TradeStatusSchema>;

// Trade schema and type
export const TradeSchema = z.object({
	id: UUIDSchema,
	session_id: UUIDSchema,
	from_team_id: UUIDSchema,
	to_team_id: UUIDSchema,
	status: TradeStatusSchema,
	proposed_at: z.string(),
	resolved_at: z.string().optional(),
});
export type Trade = z.infer<typeof TradeSchema>;

// TradeDetail schema and type
export const TradeDetailSchema = z.object({
	id: UUIDSchema,
	trade_id: UUIDSchema,
	pick_id: UUIDSchema,
	from_team: z.boolean(),
	pick_value: z.number(),
});
export type TradeDetail = z.infer<typeof TradeDetailSchema>;

// TradeProposal schema and type
export const TradeProposalSchema = z.object({
	trade: TradeSchema,
	from_team_picks: z.array(TradeDetailSchema),
	to_team_picks: z.array(TradeDetailSchema),
	from_team_total_value: z.number(),
	to_team_total_value: z.number(),
});
export type TradeProposal = z.infer<typeof TradeProposalSchema>;
