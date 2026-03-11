import * as cheerio from "cheerio";
import type { DraftOrderData, DraftOrderEntry } from "../../types/draft-order.js";
import { normalizeSvgAbbreviation } from "../../shared/team-abbreviations.js";
import { parseRoundNumber, extractAbbrFromSvgUrl } from "./helpers.js";

interface RawPick {
  pickNumber: number;
  teamAbbr: string;
  originalTeamAbbr: string;
  isCompensatory: boolean;
}

function parseRoundPicks($: cheerio.CheerioAPI, roundEl: cheerio.Element): RawPick[] {
  const picks: RawPick[] = [];

  $(roundEl)
    .find("table > tbody > tr")
    .each((_, row) => {
      const $row = $(row);

      // Extract pick number (leading digits only)
      const pickCell = $row.find("td.pick-number").first();
      if (!pickCell.length) return;

      const pickText = pickCell.contents().first().text().trim();
      const digitMatch = pickText.match(/^(\d+)/);
      if (!digitMatch) return;
      const pickNumber = parseInt(digitMatch[1], 10);
      if (pickNumber <= 0) return;

      // Check compensatory
      const isCompensatory = pickCell
        .find('span.primary[data-balloon]')
        .toArray()
        .some((span) => {
          const balloon = $(span).attr("data-balloon") ?? "";
          return balloon.toLowerCase().includes("compensatory");
        });

      // Extract team abbreviation from logo URL
      const teamImg = $row.find("div.team-link img.logo-thumb").first();
      const teamSrc = teamImg.attr("src") ?? "";
      const teamSlug = extractAbbrFromSvgUrl(teamSrc);
      if (!teamSlug) return;
      const teamAbbr = normalizeSvgAbbreviation(teamSlug);

      // Extract original team from trade div (if present)
      const tradeImg = $row.find("div.trade img.logo-thumb").first();
      const tradeSrc = tradeImg.attr("src") ?? "";
      const tradeSlug = extractAbbrFromSvgUrl(tradeSrc);
      const originalTeamAbbr = tradeSlug ? normalizeSvgAbbreviation(tradeSlug) : teamAbbr;

      picks.push({ pickNumber, teamAbbr, originalTeamAbbr, isCompensatory });
    });

  return picks;
}

export function parseTankathonHtml(html: string, year: number): DraftOrderData {
  const $ = cheerio.load(html);
  const entries: DraftOrderEntry[] = [];
  let overallPick = 1;

  $("div.full-draft-round").each((_, roundDiv) => {
    const titleEl = $(roundDiv).find("div.round-title").first();
    const roundNumber = parseRoundNumber(titleEl.text());
    if (roundNumber === null) return;

    const picks = parseRoundPicks($, roundDiv);

    picks.forEach((pick, idx) => {
      const isTraded = pick.teamAbbr !== pick.originalTeamAbbr;
      const noteParts: string[] = [];
      if (isTraded) noteParts.push(`From ${pick.originalTeamAbbr}`);
      if (pick.isCompensatory) noteParts.push("Compensatory pick");
      const notes = noteParts.length > 0 ? noteParts.join("; ") : null;

      entries.push({
        round: roundNumber,
        pick_in_round: idx + 1,
        overall_pick: overallPick,
        team_abbreviation: pick.teamAbbr,
        original_team_abbreviation: pick.originalTeamAbbr,
        is_compensatory: pick.isCompensatory,
        notes,
      });
      overallPick++;
    });
  });

  const maxRound = entries.length > 0 ? Math.max(...entries.map((e) => e.round)) : 7;
  const source = entries.length > 0 ? "tankathon" : "template";
  const today = new Date().toISOString().slice(0, 10);

  return {
    meta: {
      version: "1.0.0",
      last_updated: today,
      sources: ["Tankathon.com"],
      source,
      draft_year: year,
      total_rounds: maxRound,
      total_picks: entries.length,
    },
    draft_order: entries,
  };
}
