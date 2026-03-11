import type { CombineData } from "../../types/combine.js";
import { makeCombineEntry } from "../../shared/combine-helpers.js";
import { normalizePosition } from "../../shared/position-normalizer.js";

const PROSPECTS: [string, string, string][] = [
  ["Arvell", "Reese", "LB"],
  ["Rueben", "Bain Jr.", "EDGE"],
  ["Caleb", "Downs", "S"],
  ["Fernando", "Mendoza", "QB"],
  ["David", "Bailey", "EDGE"],
  ["Francis", "Mauigoa", "OT"],
  ["Carnell", "Tate", "WR"],
  ["Spencer", "Fano", "OT"],
  ["Jeremiyah", "Love", "RB"],
  ["Jordyn", "Tyson", "WR"],
  ["Mansoor", "Delane", "CB"],
  ["Makai", "Lemon", "WR"],
  ["Sonny", "Styles", "LB"],
  ["Jermod", "McCoy", "CB"],
  ["Keldric", "Faulk", "EDGE"],
  ["Peter", "Woods", "DT"],
  ["Kenyon", "Sadiq", "TE"],
  ["Vega", "Ioane", "OG"],
  ["Denzel", "Boston", "WR"],
  ["Cashius", "Howell", "EDGE"],
  ["Avieon", "Terrell", "CB"],
  ["Kadyn", "Proctor", "OT"],
  ["Caleb", "Lomu", "OT"],
  ["CJ", "Allen", "LB"],
  ["Kayden", "McDonald", "DT"],
  ["KC", "Concepcion", "WR"],
  ["Ty", "Simpson", "QB"],
  ["TJ", "Parker", "EDGE"],
  ["Caleb", "Banks", "DT"],
  ["Brandon", "Cisse", "CB"],
  ["Akheem", "Mesidor", "EDGE"],
  ["Monroe", "Freeling", "OT"],
  ["Colton", "Hood", "CB"],
  ["Emmanuel", "McNeil-Warren", "S"],
  ["Anthony", "Hill Jr.", "LB"],
  ["Emmanuel", "Pregnon", "OG"],
  ["Lee", "Hunter", "DT"],
  ["Dillon", "Thieneman", "S"],
  ["Blake", "Miller", "OT"],
  ["R. Mason", "Thomas", "EDGE"],
  ["Chris", "Bell", "WR"],
  ["Christen", "Miller", "DT"],
  ["Zion", "Young", "EDGE"],
  ["Gennings", "Dunker", "OT"],
  ["Chris", "Johnson", "CB"],
  ["Zachariah", "Branch", "WR"],
  ["Keith", "Abney II", "CB"],
  ["D'Angelo", "Ponds", "CB"],
  ["Germie", "Bernard", "WR"],
  ["Max", "Iheanachor", "OT"],
];

export function generateTemplateCombine(year: number): CombineData {
  const entries = PROSPECTS.map(([first, last, pos]) => makeCombineEntry(first, last, normalizePosition(pos), year));

  return {
    meta: {
      source: "template",
      description: `${year} NFL Combine template data`,
      year,
      generated_at: new Date().toISOString(),
      player_count: entries.length,
      entry_count: entries.length,
    },
    combine_results: entries,
  };
}
