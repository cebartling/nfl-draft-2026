import { test, expect } from '../src/fixtures/test-fixture.js';
import { CreateDraftViaApi, StartSession } from '../src/screenplay/tasks/draft-tasks.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { ClickTab, ProposeTradeViaApi } from '../src/screenplay/tasks/trade-tasks.js';
import {
  SessionTradesFromApi,
  TradeCountInDatabase,
} from '../src/screenplay/questions/trade-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';
import { QueryDatabase } from '../src/screenplay/abilities/query-database.js';
import { cleanupTestDrafts } from '../src/db/cleanup.js';

interface PickRow {
  id: string;
  team_id: string;
  overall_pick: number;
}

async function firstPicksForTwoTeams(
  actor: { abilityTo: (a: typeof QueryDatabase) => InstanceType<typeof QueryDatabase> },
  draftId: string,
): Promise<{ fromPick: PickRow; toPick: PickRow }> {
  const db = actor.abilityTo(QueryDatabase);
  const result = await db.query<PickRow>(
    `SELECT id, team_id, overall_pick
     FROM draft_picks
     WHERE draft_id = $1
     ORDER BY overall_pick
     LIMIT 2`,
    [draftId],
  );
  if (result.rows.length < 2) {
    throw new Error(`Expected at least 2 picks for draft ${draftId}`);
  }
  return { fromPick: result.rows[0], toPick: result.rows[1] };
}

test.describe('Trades Tab', () => {
  test.afterAll(async () => {
    await cleanupTestDrafts();
  });

  test('Trades tab renders empty state with builder and history', async ({ actor }) => {
    const createTask = CreateDraftViaApi.named('E2E Trades Tab Empty').withRounds(1);
    await actor.attemptsTo(createTask);
    const draft = createTask.response!;

    const startTask = StartSession.forDraft(draft.id).withTimePer(30);
    await actor.attemptsTo(startTask);
    const session = startTask.sessionResponse!;

    await actor.attemptsTo(Navigate.to(`/sessions/${session.id}`));
    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // Wait for the tab bar to render
    await page.locator('#tab-trades').waitFor({ state: 'visible', timeout: 10_000 });

    // Open the Trades tab
    await actor.attemptsTo(ClickTab.withId('trades'));

    // TradeBuilder heading
    await expect(page.getByRole('heading', { name: 'Build Trade Proposal' })).toBeVisible();

    // Chart selector is present and defaults to Jimmy Johnson
    const chartSelect = page.locator('#trade-builder-chart-type');
    await expect(chartSelect).toBeVisible();
    await expect(chartSelect).toHaveValue('JimmyJohnson');

    // From/To team selectors are present
    await expect(page.locator('#from-team')).toBeVisible();
    await expect(page.locator('#to-team')).toBeVisible();

    // TradeHistory shows empty state (no proposals yet)
    await expect(page.getByRole('heading', { name: 'Trade History' })).toBeVisible();
    await expect(page.getByText('No trades yet')).toBeVisible();

    // API confirms no trades exist for this session
    const trades = await actor.asks(SessionTradesFromApi.forSession(session.id));
    expect(trades).toHaveLength(0);

    const dbCount = await actor.asks(TradeCountInDatabase.forSession(session.id));
    expect(dbCount).toBe(0);
  });

  test('Proposed trade from API appears in the Trades tab history', async ({ actor }) => {
    // Create draft + session via API. 1 round × 32 teams → pick 1 worth 3000 and pick 2 worth 2600
    // on Jimmy Johnson, which is a fair trade (13.3% difference, within 15%).
    const createTask = CreateDraftViaApi.named('E2E Trades Tab With Proposal').withRounds(1);
    await actor.attemptsTo(createTask);
    const draft = createTask.response!;

    const startTask = StartSession.forDraft(draft.id).withTimePer(30);
    await actor.attemptsTo(startTask);
    const session = startTask.sessionResponse!;

    // Pull two adjacent picks owned by different teams (guaranteed in round 1)
    const { fromPick, toPick } = await firstPicksForTwoTeams(actor, draft.id);

    // Propose the trade via the backend endpoint we just wired up
    const proposeTask = ProposeTradeViaApi.with({
      sessionId: session.id,
      fromTeamId: fromPick.team_id,
      toTeamId: toPick.team_id,
      fromTeamPicks: [fromPick.id],
      toTeamPicks: [toPick.id],
      chartType: 'JimmyJohnson',
    });
    await actor.attemptsTo(proposeTask);
    expect(proposeTask.response!.trade.status).toBe('Proposed');
    expect(proposeTask.response!.from_team_picks).toEqual([fromPick.id]);
    expect(proposeTask.response!.to_team_picks).toEqual([toPick.id]);

    // API echoes the proposal on the session-scoped list endpoint
    const apiTrades = await actor.asks(SessionTradesFromApi.forSession(session.id));
    expect(apiTrades).toHaveLength(1);
    expect(apiTrades[0].trade.id).toBe(proposeTask.response!.trade.id);

    // DB persistence sanity check
    const dbCount = await actor.asks(TradeCountInDatabase.forSession(session.id));
    expect(dbCount).toBe(1);

    // Now open the UI and verify the proposal renders in the Trades tab
    await actor.attemptsTo(Navigate.to(`/sessions/${session.id}`));
    const page = actor.abilityTo(BrowseTheWeb).getPage();

    await page.locator('#tab-trades').waitFor({ state: 'visible', timeout: 10_000 });
    await actor.attemptsTo(ClickTab.withId('trades'));

    // TradeHistory count label reads "1 trade"
    await expect(page.getByText('1 trade', { exact: true })).toBeVisible();

    // The TradeProposalCard renders with the Proposed status badge
    const tradeHistoryPanel = page.locator('#tabpanel-trades');
    await expect(
      tradeHistoryPanel.getByRole('heading', { name: 'Trade Proposal', exact: true }),
    ).toBeVisible();
    await expect(tradeHistoryPanel.getByText('Proposed').first()).toBeVisible();

    // Pick-value totals rendered by TradeProposalCard match the backend calculation
    await expect(
      tradeHistoryPanel.getByText(`Total Value: ${proposeTask.response!.trade.from_team_value}`),
    ).toBeVisible();
    await expect(
      tradeHistoryPanel.getByText(`Total Value: ${proposeTask.response!.trade.to_team_value}`),
    ).toBeVisible();
  });

  test('Rejected filter hides a Proposed trade from Trade History', async ({ actor }) => {
    const createTask = CreateDraftViaApi.named('E2E Trades Tab Filter').withRounds(1);
    await actor.attemptsTo(createTask);
    const draft = createTask.response!;

    const startTask = StartSession.forDraft(draft.id).withTimePer(30);
    await actor.attemptsTo(startTask);
    const session = startTask.sessionResponse!;

    const { fromPick, toPick } = await firstPicksForTwoTeams(actor, draft.id);

    await actor.attemptsTo(
      ProposeTradeViaApi.with({
        sessionId: session.id,
        fromTeamId: fromPick.team_id,
        toTeamId: toPick.team_id,
        fromTeamPicks: [fromPick.id],
        toTeamPicks: [toPick.id],
      }),
    );

    await actor.attemptsTo(Navigate.to(`/sessions/${session.id}`));
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.locator('#tab-trades').waitFor({ state: 'visible', timeout: 10_000 });
    await actor.attemptsTo(ClickTab.withId('trades'));

    const tradeHistoryPanel = page.locator('#tabpanel-trades');

    // Default "All" filter shows the Proposed trade
    await expect(tradeHistoryPanel.getByText('1 trade', { exact: true })).toBeVisible();

    // Clicking "Rejected" hides it
    await tradeHistoryPanel.getByRole('button', { name: 'Rejected', exact: true }).click();
    await expect(tradeHistoryPanel.getByText('0 trades', { exact: true })).toBeVisible();
    await expect(tradeHistoryPanel.getByText('No trades yet')).toBeVisible();

    // Switching back to "All" brings it back
    await tradeHistoryPanel.getByRole('button', { name: 'All', exact: true }).click();
    await expect(tradeHistoryPanel.getByText('1 trade', { exact: true })).toBeVisible();
  });
});
