import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { TeamCount } from '../src/screenplay/questions/team-questions.js';
import { PlayerCount } from '../src/screenplay/questions/player-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';

test.describe('Data Integrity', () => {
  test('teams page count matches database count', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const dbCount = await actor.asks(TeamCount.inDatabase());

    // The count is displayed in a separate element: "{count} teams"
    await expect(page.getByText(`${dbCount} teams`).first()).toBeVisible();
  });

  test('players page count matches database count', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const dbCount = await actor.asks(PlayerCount.inDatabase());

    // The count is displayed as "{filtered} of {total} players"
    await expect(page.getByText(`${dbCount} players`).first()).toBeVisible();
  });
});
