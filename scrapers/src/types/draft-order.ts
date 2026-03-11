import { z } from "zod/v4";

export const DraftOrderEntrySchema = z.object({
  round: z.number().int(),
  pick_in_round: z.number().int(),
  overall_pick: z.number().int(),
  team_abbreviation: z.string(),
  original_team_abbreviation: z.string(),
  is_compensatory: z.boolean(),
  notes: z.string().nullable(),
});

export type DraftOrderEntry = z.infer<typeof DraftOrderEntrySchema>;

export const DraftOrderMetaSchema = z.object({
  version: z.string(),
  last_updated: z.string(),
  sources: z.array(z.string()),
  source: z.string().optional(),
  draft_year: z.number().int(),
  total_rounds: z.number().int(),
  total_picks: z.number().int(),
});

export type DraftOrderMeta = z.infer<typeof DraftOrderMetaSchema>;

export const DraftOrderDataSchema = z.object({
  meta: DraftOrderMetaSchema,
  draft_order: z.array(DraftOrderEntrySchema),
});

export type DraftOrderData = z.infer<typeof DraftOrderDataSchema>;
