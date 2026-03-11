import type { CombineEntry } from "../types/combine.js";

type Measurables = Partial<
  Pick<
    CombineEntry,
    | "forty_yard_dash"
    | "bench_press"
    | "vertical_jump"
    | "broad_jump"
    | "three_cone_drill"
    | "twenty_yard_shuttle"
    | "arm_length"
    | "hand_size"
    | "wingspan"
    | "ten_yard_split"
    | "twenty_yard_split"
  >
>;

export function makeCombineEntry(
  firstName: string,
  lastName: string,
  position: string,
  year: number,
  overrides: Measurables = {},
): CombineEntry {
  return {
    first_name: firstName,
    last_name: lastName,
    position,
    source: "combine",
    year,
    forty_yard_dash: overrides.forty_yard_dash ?? null,
    bench_press: overrides.bench_press ?? null,
    vertical_jump: overrides.vertical_jump ?? null,
    broad_jump: overrides.broad_jump ?? null,
    three_cone_drill: overrides.three_cone_drill ?? null,
    twenty_yard_shuttle: overrides.twenty_yard_shuttle ?? null,
    arm_length: overrides.arm_length ?? null,
    hand_size: overrides.hand_size ?? null,
    wingspan: overrides.wingspan ?? null,
    ten_yard_split: overrides.ten_yard_split ?? null,
    twenty_yard_split: overrides.twenty_yard_split ?? null,
  };
}
