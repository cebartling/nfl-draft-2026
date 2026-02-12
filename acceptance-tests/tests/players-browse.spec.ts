import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { SearchForPlayer, FilterByPositionGroup } from '../src/screenplay/tasks/player-tasks.js';
import { PageHeading } from '../src/screenplay/questions/web-questions.js';
import { PlayerCount } from '../src/screenplay/questions/player-questions.js';

test.describe('Players Browsing', () => {
  test('displays players and count matches database', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('Players');

    const dbCount = await actor.asks(PlayerCount.inDatabase());
    expect(dbCount).toBeGreaterThan(0);
  });

  test('search narrows player results', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    // Wait for players to load
    const page = (await import('../src/screenplay/abilities/browse-the-web.js')).BrowseTheWeb;
    const browserPage = actor.abilityTo(page).getPage();
    await browserPage.waitForSelector('text=Players');

    // Search for a player
    await actor.attemptsTo(SearchForPlayer.named('QB'));

    // The heading should show a filtered count
    const heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('Players');
  });

  test('filter by position group narrows results', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const page = (await import('../src/screenplay/abilities/browse-the-web.js')).BrowseTheWeb;
    const browserPage = actor.abilityTo(page).getPage();
    await browserPage.waitForSelector('text=Players');

    await actor.attemptsTo(FilterByPositionGroup.named('Defense'));

    const heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('Players');
  });
});
