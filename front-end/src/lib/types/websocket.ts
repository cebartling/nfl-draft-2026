import { z } from 'zod';
import { UUIDSchema } from './common';

// Client → Server message schemas
const SubscribeMessageSchema = z.object({
	type: z.literal('subscribe'),
	session_id: UUIDSchema,
});

const MakePickMessageSchema = z.object({
	type: z.literal('make_pick'),
	session_id: UUIDSchema,
	player_id: UUIDSchema,
});

const ProposeTradeMessageSchema = z.object({
	type: z.literal('propose_trade'),
	session_id: UUIDSchema,
	from_team_id: UUIDSchema,
	to_team_id: UUIDSchema,
	pick_ids: z.array(UUIDSchema),
});

const PingMessageSchema = z.object({
	type: z.literal('ping'),
});

export const ClientMessageSchema = z.discriminatedUnion('type', [
	SubscribeMessageSchema,
	MakePickMessageSchema,
	ProposeTradeMessageSchema,
	PingMessageSchema,
]);
export type ClientMessage = z.infer<typeof ClientMessageSchema>;

// Server → Client message schemas
const SubscribedMessageSchema = z.object({
	type: z.literal('subscribed'),
	session_id: UUIDSchema,
});

const PickMadeMessageSchema = z.object({
	type: z.literal('pick_made'),
	session_id: UUIDSchema,
	pick_id: UUIDSchema,
	team_id: UUIDSchema,
	player_id: UUIDSchema,
	round: z.number(),
	pick_number: z.number(),
	player_name: z.string(),
	team_name: z.string(),
});

const ClockUpdateMessageSchema = z.object({
	type: z.literal('clock_update'),
	session_id: UUIDSchema,
	time_remaining: z.number(),
	current_pick_number: z.number(),
});

const DraftStatusMessageSchema = z.object({
	type: z.literal('draft_status'),
	session_id: UUIDSchema,
	status: z.string(),
});

const TradeProposedMessageSchema = z.object({
	type: z.literal('trade_proposed'),
	session_id: UUIDSchema,
	trade_id: UUIDSchema,
	from_team_id: UUIDSchema,
	to_team_id: UUIDSchema,
	from_team_name: z.string(),
	to_team_name: z.string(),
	from_team_picks: z.array(UUIDSchema),
	to_team_picks: z.array(UUIDSchema),
	from_team_value: z.number(),
	to_team_value: z.number(),
});

const TradeExecutedMessageSchema = z.object({
	type: z.literal('trade_executed'),
	session_id: UUIDSchema,
	trade_id: UUIDSchema,
	from_team_id: UUIDSchema,
	to_team_id: UUIDSchema,
});

const TradeRejectedMessageSchema = z.object({
	type: z.literal('trade_rejected'),
	session_id: UUIDSchema,
	trade_id: UUIDSchema,
	rejecting_team_id: UUIDSchema,
});

const ErrorMessageSchema = z.object({
	type: z.literal('error'),
	message: z.string(),
});

const PongMessageSchema = z.object({
	type: z.literal('pong'),
});

export const ServerMessageSchema = z.discriminatedUnion('type', [
	SubscribedMessageSchema,
	PickMadeMessageSchema,
	ClockUpdateMessageSchema,
	DraftStatusMessageSchema,
	TradeProposedMessageSchema,
	TradeExecutedMessageSchema,
	TradeRejectedMessageSchema,
	ErrorMessageSchema,
	PongMessageSchema,
]);
export type ServerMessage = z.infer<typeof ServerMessageSchema>;
