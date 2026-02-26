import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';
import { ExpandDivision } from '../src/screenplay/tasks/team-page-tasks.js';
import { TeamDetails } from '../src/screenplay/questions/team-questions.js';
import { QueryDatabase } from '../src/screenplay/abilities/query-database.js';

test.describe('Team Detail Page — Extended', () => {
  test('displays team name and abbreviation badge', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await actor.attemptsTo(ExpandDivision.named('AFC East'));

    await page.getByText('Buffalo Bills').first().click();
    await page.waitForURL(/\/teams\/[0-9a-f-]+/);
    await page.waitForLoadState('networkidle');

    // Verify the h1 heading contains both city and team name
    await expect(page.getByRole('heading', { level: 1 })).toContainText(
      'Buffalo',
    );
    await expect(page.getByRole('heading', { level: 1 })).toContainText(
      'Bills',
    );

    // Verify abbreviation badge (use exact match to avoid matching "Buffalo")
    await expect(page.getByText('BUF', { exact: true }).first()).toBeVisible();
  });

  test('displays season statistics section', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await actor.attemptsTo(ExpandDivision.named('AFC East'));

    await page.getByText('Buffalo Bills').first().click();
    await page.waitForURL(/\/teams\/[0-9a-f-]+/);
    await page.waitForLoadState('networkidle');

    // Verify season heading is visible
    await expect(
      page.getByRole('heading', { name: /Season/i }),
    ).toBeVisible();

    // Verify stat labels are present
    await expect(page.getByText('Wins')).toBeVisible();
    await expect(page.getByText('Losses')).toBeVisible();
  });

  test('displays team needs section', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await actor.attemptsTo(ExpandDivision.named('AFC East'));

    await page.getByText('Buffalo Bills').first().click();
    await page.waitForURL(/\/teams\/[0-9a-f-]+/);
    await page.waitForLoadState('networkidle');

    // TeamNeeds component should render — check for the heading or content
    await expect(
      page.getByRole('heading', { name: /Team Needs|Needs/i }),
    ).toBeVisible({ timeout: 10000 });
  });

  test('season statistics match database record', async ({ actor }) => {
    // Look up Buffalo Bills in DB
    const team = await actor.asks(TeamDetails.inDatabaseFor('BUF'));
    expect(team).not.toBeNull();

    const db = actor.abilityTo(QueryDatabase);
    const season = await db.queryOne<{
      wins: number;
      losses: number;
      draft_position: number;
    }>(
      'SELECT wins, losses, draft_position FROM team_seasons WHERE team_id = $1 AND season_year = 2025',
      [team!.id],
    );

    await actor.attemptsTo(Navigate.to(`/teams/${team!.id}`));

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    if (season) {
      // Verify the wins number appears on the page
      await expect(
        page.getByText(String(season.wins)).first(),
      ).toBeVisible();
      // Verify the losses number appears on the page
      await expect(
        page.getByText(String(season.losses)).first(),
      ).toBeVisible();
    }
  });

  test('displays Draft Picks section with empty state', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await actor.attemptsTo(ExpandDivision.named('AFC East'));

    await page.getByText('Buffalo Bills').first().click();
    await page.waitForURL(/\/teams\/[0-9a-f-]+/);
    await page.waitForLoadState('networkidle');

    // Draft Picks heading
    await expect(
      page.getByRole('heading', { name: 'Draft Picks' }),
    ).toBeVisible();

    // Empty state message
    await expect(
      page.getByText(/No draft picks available/i),
    ).toBeVisible();
  });
});
