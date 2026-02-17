import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { SearchForPlayer, FilterByPositionGroup } from '../src/screenplay/tasks/player-tasks.js';
import { PageHeading, CurrentUrl } from '../src/screenplay/questions/web-questions.js';
import { PlayerCount } from '../src/screenplay/questions/player-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';

test.describe('Players Browsing', () => {
  test('displays players and count matches database', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('Players');

    const dbCount = await actor.asks(PlayerCount.inDatabase());
    expect(dbCount).toBeGreaterThan(0);

    // The UI shows "N of M players" — the total M should match DB count
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await expect(page.getByText(`of ${dbCount} players`).first()).toBeVisible();
  });

  test('search narrows player results', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // Wait for players to load (non-zero total in "N of M players")
    const countText = page.locator('.text-sm.text-gray-600').first();
    await expect(countText).toContainText(/[1-9]\d* of [1-9]\d* players/);
    const initialText = (await countText.textContent()) ?? '';
    const initialMatch = initialText.match(/(\d+)\s+of\s+(\d+)\s+players/);
    expect(initialMatch).not.toBeNull();
    const initialCount = parseInt(initialMatch![1], 10);
    expect(initialCount).toBeGreaterThan(0);

    // Search by college name (search filters on first_name, last_name, college)
    await actor.attemptsTo(SearchForPlayer.named('Ohio State'));

    // The active filter badge should appear
    await expect(page.getByText('Search: "Ohio State"').first()).toBeVisible();

    // The filtered count should be less than the initial count
    await expect(countText).not.toContainText(`${initialCount} of`);
    const filteredText = (await countText.textContent()) ?? '';
    const filteredMatch = filteredText.match(/(\d+)\s+of/);
    const filteredCount = filteredMatch ? parseInt(filteredMatch[1], 10) : 0;
    expect(filteredCount).toBeLessThan(initialCount);
    expect(filteredCount).toBeGreaterThan(0);
  });

  test('filter by position group narrows results', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // Wait for players to load (non-zero total in "N of M players")
    const countText = page.locator('.text-sm.text-gray-600').first();
    await expect(countText).toContainText(/[1-9]\d* of [1-9]\d* players/);
    const initialText = (await countText.textContent()) ?? '';
    const initialMatch = initialText.match(/(\d+)\s+of\s+(\d+)\s+players/);
    expect(initialMatch).not.toBeNull();
    const initialCount = parseInt(initialMatch![1], 10);
    expect(initialCount).toBeGreaterThan(0);

    // Filter by defense group
    await actor.attemptsTo(FilterByPositionGroup.named('Defense'));

    // The active filter badge should appear
    await expect(page.getByText('Group: defense').first()).toBeVisible();

    // The filtered count should be less than the initial count
    await expect(countText).not.toContainText(`${initialCount} of`);
    const filteredText = (await countText.textContent()) ?? '';
    const filteredMatch = filteredText.match(/(\d+)\s+of/);
    const filteredCount = filteredMatch ? parseInt(filteredMatch[1], 10) : 0;
    expect(filteredCount).toBeLessThan(initialCount);
    expect(filteredCount).toBeGreaterThan(0);
  });

  test('player cards are visible after load', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('[role="button"]', { timeout: 10000 });

    const playerCards = page.locator('[role="button"]');
    await expect(playerCards.first()).toBeVisible();
  });

  test('position badges are visible on player cards', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('[role="button"]', { timeout: 10000 });

    const positionBadges = page.locator(
      '[role="button"] span:text-matches("^(QB|RB|WR|TE|OT|OG|C|DE|DT|LB|CB|S|K|P)$")',
    );
    await expect(positionBadges.first()).toBeVisible();
  });

  test('navigate to player detail from card click', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('[role="button"]', { timeout: 10000 });

    await page.locator('[role="button"]').first().click();
    await page.waitForURL(/\/players\/[a-f0-9-]+/);

    const url = await actor.asks(CurrentUrl.value());
    expect(url).toMatch(/\/players\/[a-f0-9-]+/);
  });
});
