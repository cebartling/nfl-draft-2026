import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { PageHeading, CurrentUrl } from '../src/screenplay/questions/web-questions.js';
import { TeamCount, TeamDetails } from '../src/screenplay/questions/team-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';
import { ExpandDivision, ToggleTeamGrouping } from '../src/screenplay/tasks/team-page-tasks.js';

test.describe('Teams Browsing', () => {
  test('displays all 32 teams and count matches database', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('Teams');

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const dbCount = await actor.asks(TeamCount.inDatabase());
    expect(dbCount).toBe(32);

    // The count "32 teams" is displayed in a separate element from the h1
    await expect(page.getByText(`${dbCount} teams`).first()).toBeVisible();
  });

  test('can navigate to team detail page', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // Teams are in collapsible division accordions — expand one first
    const divisionButton = page.getByText('AFC East').first();
    await divisionButton.click();

    // Click on a team card (the card shows the full team name)
    await page.getByText('Buffalo Bills').first().click();
    await page.waitForURL(/\/teams\/[0-9a-f-]+/);

    const url = await actor.asks(CurrentUrl.value());
    expect(url).toMatch(/\/teams\/[0-9a-f-]+/);
  });

  test('team detail matches database record', async ({ actor }) => {
    const dbTeam = await actor.asks(TeamDetails.inDatabaseFor('BUF'));
    expect(dbTeam).not.toBeNull();
    expect(dbTeam!.name).toContain('Bills');
    expect(dbTeam!.city).toBe('Buffalo');
    expect(dbTeam!.conference).toMatch(/AFC/);

    // Navigate to the team via the UI
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // Expand the AFC East division
    await page.getByText('AFC East').first().click();

    // Click on the Buffalo Bills team card
    await page.getByText('Buffalo Bills').first().click();
    await page.waitForURL(/\/teams\/[0-9a-f-]+/);

    const url = await actor.asks(CurrentUrl.value());
    expect(url).toContain(dbTeam!.id);
  });

  test('expanding division reveals team cards with names', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await actor.attemptsTo(ExpandDivision.named('AFC East'));

    const expandedGrid = page.locator('.grid');
    await expect(expandedGrid.first()).toBeVisible({ timeout: 5000 });

    const teamNames = expandedGrid.locator('h3');
    await expect(teamNames.first()).toBeVisible();
  });

  test('switch between Conference and Division grouping', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await expect(page.locator('button', { hasText: 'Conference' })).toBeVisible();
    await expect(page.locator('button', { hasText: 'Division' })).toBeVisible();

    await actor.attemptsTo(ToggleTeamGrouping.to('Division'));

    await expect(page.locator('h2').first()).toBeVisible();
  });

  test('conference badges AFC and NFC are visible', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    const conferenceText = page.locator('text=/AFC|NFC/');
    await expect(conferenceText.first()).toBeVisible({ timeout: 10000 });
  });
});
