import { z } from "zod/v4";

export const CombineEntrySchema = z.object({
  first_name: z.string(),
  last_name: z.string(),
  position: z.string(),
  source: z.string(),
  year: z.number().int(),
  forty_yard_dash: z.number().nullable(),
  bench_press: z.number().int().nullable(),
  vertical_jump: z.number().nullable(),
  broad_jump: z.number().int().nullable(),
  three_cone_drill: z.number().nullable(),
  twenty_yard_shuttle: z.number().nullable(),
  arm_length: z.number().nullable(),
  hand_size: z.number().nullable(),
  wingspan: z.number().nullable(),
  ten_yard_split: z.number().nullable(),
  twenty_yard_split: z.number().nullable(),
});

export type CombineEntry = z.infer<typeof CombineEntrySchema>;

export const CombineMetaSchema = z.object({
  source: z.string(),
  description: z.string(),
  year: z.number().int(),
  generated_at: z.string(),
  player_count: z.number().int(),
  entry_count: z.number().int(),
});

export type CombineMeta = z.infer<typeof CombineMetaSchema>;

export const CombineDataSchema = z.object({
  meta: CombineMetaSchema,
  combine_results: z.array(CombineEntrySchema),
});

export type CombineData = z.infer<typeof CombineDataSchema>;
