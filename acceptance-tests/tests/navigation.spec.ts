import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { CurrentUrl, PageHeading } from '../src/screenplay/questions/web-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';

test.describe('Navigation', () => {
  test('navbar links navigate between pages', async ({ actor }) => {
    // Start at home
    await actor.attemptsTo(Navigate.toHomePage());
    let heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('NFL Draft');

    // Navigate to Drafts
    await actor.attemptsTo(Navigate.toDrafts());
    heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('Drafts');
    let url = await actor.asks(CurrentUrl.value());
    expect(url).toContain('/drafts');

    // Navigate to Players
    await actor.attemptsTo(Navigate.toPlayers());
    heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('Players');
    url = await actor.asks(CurrentUrl.value());
    expect(url).toContain('/players');

    // Navigate to Teams
    await actor.attemptsTo(Navigate.toTeams());
    heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('Teams');
    url = await actor.asks(CurrentUrl.value());
    expect(url).toContain('/teams');
  });

  test('mobile menu toggles navigation links', async ({ actor }) => {
    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });

    await actor.attemptsTo(Navigate.toHomePage());

    // Desktop nav should be hidden on mobile
    const desktopNav = page.locator('.hidden.md\\:flex');
    await expect(desktopNav).not.toBeVisible();

    // Click hamburger menu
    const menuButton = page.getByRole('button', { name: 'Menu' });
    await expect(menuButton).toBeVisible();
    await menuButton.click();

    // Mobile menu links should be visible
    const mobileHome = page.locator('.md\\:hidden >> a[href="/"]');
    await expect(mobileHome.first()).toBeVisible();

    // Click a mobile nav link
    const mobileDrafts = page.locator('.md\\:hidden >> a[href="/drafts"]');
    await mobileDrafts.first().click();

    const url = await actor.asks(CurrentUrl.value());
    expect(url).toContain('/drafts');
  });
});
