import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { PageHeading, CurrentUrl } from '../src/screenplay/questions/web-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';

test.describe('Player Detail Page', () => {
  test('displays player heading after navigation', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('[role="button"]', { timeout: 10000 });

    await page.locator('[role="button"]').first().click();
    await page.waitForURL(/\/players\/[a-f0-9-]+/);

    const heading = await actor.asks(PageHeading.text());
    expect(heading.length).toBeGreaterThan(0);
  });

  test('displays player stats including Height', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('[role="button"]', { timeout: 10000 });

    await page.locator('[role="button"]').first().click();
    await page.waitForURL(/\/players\/[a-f0-9-]+/);

    await expect(page.getByText('Height').first()).toBeVisible();
  });

  test('browser back returns to players list', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toPlayers());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('[role="button"]', { timeout: 10000 });

    await page.locator('[role="button"]').first().click();
    await page.waitForURL(/\/players\/[a-f0-9-]+/);

    await page.goBack();
    await page.waitForURL(/\/players$/);

    const url = await actor.asks(CurrentUrl.value());
    expect(url).toMatch(/\/players$/);
  });
});
