const POSITION_MAP: Record<string, string> = {
  QB: "QB",
  RB: "RB",
  HB: "RB",
  FB: "RB",
  WR: "WR",
  TE: "TE",
  OT: "OT",
  T: "OT",
  OG: "OG",
  G: "OG",
  IOL: "OG",
  OL: "OG",
  C: "C",
  DE: "DE",
  EDGE: "DE",
  "EDGE/LB": "DE",
  "LB/EDGE": "DE",
  DT: "DT",
  DL: "DT",
  NT: "DT",
  LB: "LB",
  OLB: "LB",
  ILB: "LB",
  MLB: "LB",
  CB: "CB",
  S: "S",
  SS: "S",
  FS: "S",
  DB: "S",
  SAF: "S",
  K: "K",
  P: "P",
};

export function normalizePosition(pos: string): string {
  const upper = pos.trim().toUpperCase();
  return POSITION_MAP[upper] ?? upper;
}
