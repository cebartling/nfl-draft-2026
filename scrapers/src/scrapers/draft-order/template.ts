import type { DraftOrderData, DraftOrderEntry } from "../../types/draft-order.js";

const TEAMS_BY_ROUND1_ORDER = [
  "TEN", "CLE", "NYG", "NE", "JAX", "LV", "NYJ", "CAR", "NO", "CHI",
  "SF", "DAL", "MIA", "IND", "ATL", "ARI", "CIN", "SEA", "TB", "DEN",
  "PIT", "LAC", "GB", "HOU", "MIN", "LAR", "BAL", "WAS", "BUF", "DET",
  "PHI", "KC",
];

const COMP_TEAMS_BY_ROUND: Record<number, string[]> = {
  3: ["NE", "SF", "LAR", "KC"],
  4: ["DAL", "CHI", "BAL", "MIN"],
  5: ["NYG", "CLE", "NO", "DET"],
  6: ["JAX", "LV", "TB", "HOU"],
  7: ["CAR", "CIN", "SEA", "PHI"],
};

export function generateTemplateDraftOrder(year: number): DraftOrderData {
  const entries: DraftOrderEntry[] = [];
  let overallPick = 1;

  for (let round = 1; round <= 7; round++) {
    for (const team of TEAMS_BY_ROUND1_ORDER) {
      entries.push({
        round,
        pick_in_round: 0, // fixed below
        overall_pick: overallPick,
        team_abbreviation: team,
        original_team_abbreviation: team,
        is_compensatory: false,
        notes: null,
      });
      overallPick++;
    }

    const compTeams = COMP_TEAMS_BY_ROUND[round] ?? [];
    for (const team of compTeams) {
      entries.push({
        round,
        pick_in_round: 0, // fixed below
        overall_pick: overallPick,
        team_abbreviation: team,
        original_team_abbreviation: team,
        is_compensatory: true,
        notes: "Compensatory pick",
      });
      overallPick++;
    }
  }

  // Fix pick_in_round sequentially per round
  let currentRound = 0;
  let pickInRound = 0;
  for (const entry of entries) {
    if (entry.round !== currentRound) {
      currentRound = entry.round;
      pickInRound = 0;
    }
    pickInRound++;
    entry.pick_in_round = pickInRound;
  }

  const today = new Date().toISOString().slice(0, 10);

  return {
    meta: {
      version: "1.0.0",
      last_updated: today,
      sources: ["Template (edit manually)"],
      source: "template",
      draft_year: year,
      total_rounds: 7,
      total_picks: entries.length,
    },
    draft_order: entries,
  };
}
