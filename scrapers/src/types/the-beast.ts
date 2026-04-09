import { z } from "zod/v4";

/**
 * Year-by-year college statistics row. Shape varies by position; we
 * keep this loose so the parser doesn't have to know offense vs. defense.
 */
export const CollegeStatRowSchema = z.object({
  year: z.string(), // "2025" or "2025: (16/16)"
  notes: z.string().nullable(),
});

export type CollegeStatRow = z.infer<typeof CollegeStatRowSchema>;

/** Parsed combine / pro-day measurables row. */
export const MeasurablesSchema = z.object({
  height_raw: z.string().nullable(), // raw 4-digit beast format e.g. "6046"
  weight_pounds: z.number().int().nullable(),
  hand_size: z.number().nullable(), // inches
  arm_length: z.number().nullable(),
  wingspan: z.number().nullable(),
  forty_yard_dash: z.number().nullable(),
  twenty_yard_split: z.number().nullable(),
  ten_yard_split: z.number().nullable(),
  vertical_jump: z.number().nullable(),
  broad_jump: z.number().int().nullable(), // inches
  twenty_yard_shuttle: z.number().nullable(),
  three_cone_drill: z.number().nullable(),
  bench_press: z.number().int().nullable(),
});

export type Measurables = z.infer<typeof MeasurablesSchema>;

export const BeastProspectSchema = z.object({
  // Identity
  position: z.string(),
  position_rank: z.number().int(),
  overall_rank: z.number().int().nullable(),
  first_name: z.string(),
  last_name: z.string(),
  school: z.string(),

  // Header / metadata
  grade_tier: z.string().nullable(), // "1st round", "4th-5th", "7th-FA", "FA"
  year_class: z.string().nullable(), // "4JR", "5SR", "6SR", "7SR"
  birthday: z.string().nullable(), // ISO date "2003-10-01"
  age: z.number().nullable(),
  jersey_number: z.string().nullable(),

  // Physical (parsed from header table; may also exist in combine)
  height_inches: z.number().int().nullable(),
  weight_pounds: z.number().int().nullable(),
  height_raw: z.string().nullable(),

  // Workouts (top-level convenience for the loader; full tables in combine/pro_day)
  forty_yard_dash: z.number().nullable(),
  ten_yard_split: z.number().nullable(),
  hand_size: z.number().nullable(),
  arm_length: z.number().nullable(),
  wingspan: z.number().nullable(),

  combine: MeasurablesSchema.nullable(),
  pro_day: MeasurablesSchema.nullable(),

  // Stats and prose
  college_stats: z.array(CollegeStatRowSchema),
  background: z.string().nullable(),
  strengths: z.array(z.string()),
  weaknesses: z.array(z.string()),
  summary: z.string().nullable(),
  nfl_comparison: z.string().nullable(),
});

export type BeastProspect = z.infer<typeof BeastProspectSchema>;

export const BeastMetaSchema = z.object({
  version: z.string(),
  source: z.string(), // "the-beast-2026"
  source_url: z.string(),
  draft_year: z.number().int(),
  scraped_at: z.string(), // YYYY-MM-DD
  total_prospects: z.number().int(),
});

export type BeastMeta = z.infer<typeof BeastMetaSchema>;

export const BeastDataSchema = z.object({
  meta: BeastMetaSchema,
  prospects: z.array(BeastProspectSchema),
});

export type BeastData = z.infer<typeof BeastDataSchema>;
