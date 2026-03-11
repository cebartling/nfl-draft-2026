import { z } from "zod/v4";

export const RankingEntrySchema = z.object({
  rank: z.number().int(),
  first_name: z.string(),
  last_name: z.string(),
  position: z.string(),
  school: z.string(),
  height_inches: z.number().int().nullable(),
  weight_pounds: z.number().int().nullable(),
});

export type RankingEntry = z.infer<typeof RankingEntrySchema>;

export const RankingMetaSchema = z.object({
  version: z.string(),
  source: z.string(),
  source_url: z.string(),
  draft_year: z.number().int(),
  scraped_at: z.string(),
  total_prospects: z.number().int(),
});

export type RankingMeta = z.infer<typeof RankingMetaSchema>;

export const RankingDataSchema = z.object({
  meta: RankingMetaSchema,
  rankings: z.array(RankingEntrySchema),
});

export type RankingData = z.infer<typeof RankingDataSchema>;
