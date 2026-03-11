import { describe, it, expect } from "vitest";
import { parseTankathonHtml } from "../../../src/scrapers/draft-order/parser.js";

function makeRoundHtml(roundTitle: string, rows: string): string {
  return `<div class="full-draft-round full-draft-round-nfl">
    <div class="round-title">${roundTitle}</div>
    <table><tbody>${rows}</tbody></table>
  </div>`;
}

function makePickRow(pickNum: number, teamSlug: string): string {
  return `<tr>
    <td class="pick-number">${pickNum}</td>
    <td>
      <div class="team-link"><a href=""><img class="logo-thumb" src="/img/nfl/${teamSlug}.svg"></a></div>
    </td>
  </tr>`;
}

function makeTradedPickRow(pickNum: number, teamSlug: string, originalSlug: string): string {
  return `<tr>
    <td class="pick-number">${pickNum}</td>
    <td>
      <div class="team-link"><a href=""><img class="logo-thumb" src="/img/nfl/${teamSlug}.svg"></a></div>
      <div class="trade"><a href=""><img class="logo-thumb" src="/img/nfl/${originalSlug}.svg"></a></div>
    </td>
  </tr>`;
}

function makeCompPickRow(pickNum: number, teamSlug: string): string {
  return `<tr>
    <td class="pick-number">${pickNum}
      <span class="primary" data-balloon="Compensatory pick">C</span>
    </td>
    <td>
      <div class="team-link"><a href=""><img class="logo-thumb" src="/img/nfl/${teamSlug}.svg"></a></div>
    </td>
  </tr>`;
}

describe("parseTankathonHtml", () => {
  it("parses a basic round with 3 picks", () => {
    const rows = [makePickRow(1, "ten"), makePickRow(2, "cle"), makePickRow(3, "nyg")].join("\n");
    const html = `<html><body>${makeRoundHtml("1st Round", rows)}</body></html>`;

    const data = parseTankathonHtml(html, 2026);
    expect(data.draft_order).toHaveLength(3);
    expect(data.meta.source).toBe("tankathon");
    expect(data.meta.total_picks).toBe(3);

    expect(data.draft_order[0].round).toBe(1);
    expect(data.draft_order[0].pick_in_round).toBe(1);
    expect(data.draft_order[0].overall_pick).toBe(1);
    expect(data.draft_order[0].team_abbreviation).toBe("TEN");
    expect(data.draft_order[0].original_team_abbreviation).toBe("TEN");
    expect(data.draft_order[0].is_compensatory).toBe(false);
    expect(data.draft_order[0].notes).toBeNull();

    expect(data.draft_order[2].team_abbreviation).toBe("NYG");
    expect(data.draft_order[2].overall_pick).toBe(3);
  });

  it("parses a traded pick", () => {
    const rows = makeTradedPickRow(5, "lv", "atl");
    const html = `<html><body>${makeRoundHtml("1st Round", rows)}</body></html>`;

    const data = parseTankathonHtml(html, 2026);
    expect(data.draft_order).toHaveLength(1);

    const entry = data.draft_order[0];
    expect(entry.team_abbreviation).toBe("LV");
    expect(entry.original_team_abbreviation).toBe("ATL");
    expect(entry.is_compensatory).toBe(false);
    expect(entry.notes).toBe("From ATL");
  });

  it("parses a compensatory pick", () => {
    const rows = makeCompPickRow(33, "ne");
    const html = `<html><body>${makeRoundHtml("3rd Round", rows)}</body></html>`;

    const data = parseTankathonHtml(html, 2026);
    expect(data.draft_order).toHaveLength(1);

    const entry = data.draft_order[0];
    expect(entry.team_abbreviation).toBe("NE");
    expect(entry.original_team_abbreviation).toBe("NE");
    expect(entry.is_compensatory).toBe(true);
    expect(entry.notes).toBe("Compensatory pick");
  });

  it("parses a traded compensatory pick", () => {
    const rows = `<tr>
      <td class="pick-number">35
        <span class="primary" data-balloon="Compensatory pick">C</span>
      </td>
      <td>
        <div class="team-link"><a href=""><img class="logo-thumb" src="/img/nfl/dal.svg"></a></div>
        <div class="trade"><a href=""><img class="logo-thumb" src="/img/nfl/sf.svg"></a></div>
      </td>
    </tr>`;
    const html = `<html><body>${makeRoundHtml("3rd Round", rows)}</body></html>`;

    const data = parseTankathonHtml(html, 2026);
    expect(data.draft_order).toHaveLength(1);

    const entry = data.draft_order[0];
    expect(entry.team_abbreviation).toBe("DAL");
    expect(entry.original_team_abbreviation).toBe("SF");
    expect(entry.is_compensatory).toBe(true);
    expect(entry.notes).toBe("From SF; Compensatory pick");
  });

  it("parses multiple rounds with sequential overall picks", () => {
    const round1 = [makePickRow(1, "ten"), makePickRow(2, "cle")].join("\n");
    const round2 = [makePickRow(33, "cle"), makePickRow(34, "ten")].join("\n");
    const html = `<html><body>${makeRoundHtml("1st Round", round1)}${makeRoundHtml("2nd Round", round2)}</body></html>`;

    const data = parseTankathonHtml(html, 2026);
    expect(data.draft_order).toHaveLength(4);
    expect(data.meta.total_rounds).toBe(2);

    // Round 1
    expect(data.draft_order[0].round).toBe(1);
    expect(data.draft_order[0].overall_pick).toBe(1);
    expect(data.draft_order[0].pick_in_round).toBe(1);
    expect(data.draft_order[1].round).toBe(1);
    expect(data.draft_order[1].overall_pick).toBe(2);
    expect(data.draft_order[1].pick_in_round).toBe(2);

    // Round 2: pick_in_round resets, overall_pick continues
    expect(data.draft_order[2].round).toBe(2);
    expect(data.draft_order[2].overall_pick).toBe(3);
    expect(data.draft_order[2].pick_in_round).toBe(1);
    expect(data.draft_order[3].round).toBe(2);
    expect(data.draft_order[3].overall_pick).toBe(4);
    expect(data.draft_order[3].pick_in_round).toBe(2);
  });

  it("returns empty draft order for HTML without round containers", () => {
    const html = "<html><body><p>Nothing here</p></body></html>";
    const data = parseTankathonHtml(html, 2026);
    expect(data.draft_order).toHaveLength(0);
    expect(data.meta.source).toBe("template");
    expect(data.meta.total_picks).toBe(0);
  });

  it("normalizes wsh to WAS", () => {
    const rows = makePickRow(28, "wsh");
    const html = `<html><body>${makeRoundHtml("1st Round", rows)}</body></html>`;

    const data = parseTankathonHtml(html, 2026);
    expect(data.draft_order[0].team_abbreviation).toBe("WAS");
  });

  it("normalizes jac to JAX", () => {
    const rows = makePickRow(5, "jac");
    const html = `<html><body>${makeRoundHtml("1st Round", rows)}</body></html>`;

    const data = parseTankathonHtml(html, 2026);
    expect(data.draft_order[0].team_abbreviation).toBe("JAX");
  });
});
