import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { SearchForPlayer, FilterByPositionGroup } from '../src/screenplay/tasks/player-tasks.js';
import { SelectPosition, ClearAllFilters } from '../src/screenplay/tasks/prospect-tasks.js';
import { PageHeading, CurrentUrl } from '../src/screenplay/questions/web-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';

test.describe('Prospects Page', () => {
  test('should load prospects page with heading', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const heading = await actor.asks(PageHeading.text());
    expect(heading).toMatch(/prospect rankings/i);
  });

  test('should display ranked prospect count and source count in header', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    const headerStats = page.locator('text=/\\d+ ranked prospects/');
    await expect(headerStats).toBeVisible({ timeout: 10000 });

    const sourcesText = page.locator('text=/\\d+ sources/');
    await expect(sourcesText).toBeVisible();
  });

  test('should display rankings table with rows', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    await expect(page.locator('th', { hasText: '#' })).toBeVisible();
    await expect(page.locator('th', { hasText: 'Player' })).toBeVisible();
    await expect(page.locator('th', { hasText: 'Pos' })).toBeVisible();
    await expect(page.locator('th', { hasText: 'Big Board Rankings' })).toBeVisible();

    const rows = page.locator('tbody tr');
    await expect(rows.first()).toBeVisible();
  });

  test('should display ranking badges for prospects', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    const badges = page.locator('.bg-purple-100');
    await expect(badges.first()).toBeVisible({ timeout: 10000 });
  });

  test('should display consensus rank numbers', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    const firstRow = page.locator('tbody tr').first();
    const rankCell = firstRow.locator('td').first();
    await expect(rankCell).toContainText('1');
  });

  test('should filter prospects by search query', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    const firstRowName = await page.locator('tbody tr').first().locator('td').nth(1).textContent();
    const searchTerm = firstRowName?.trim().split(/\s+/).pop() || '';

    if (searchTerm) {
      await actor.attemptsTo(SearchForPlayer.named(searchTerm));

      await expect(page.locator('text=/Search:/i')).toBeVisible();

      const resultText = await page.locator('tbody').textContent();
      expect(resultText?.toLowerCase()).toContain(searchTerm.toLowerCase());
    }
  });

  test('should filter prospects by position', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    await actor.attemptsTo(SelectPosition.named('QB'));

    await expect(page.locator('text=/Position: QB/i')).toBeVisible();

    const rows = page.locator('tbody tr');
    const rowCount = await rows.count();

    if (rowCount > 0) {
      for (let i = 0; i < Math.min(rowCount, 5); i++) {
        const posCell = rows.nth(i).locator('td').nth(2);
        await expect(posCell).toContainText('QB');
      }
    }
  });

  test('should clear filters', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    await actor.attemptsTo(SearchForPlayer.named('test-nonexistent-xyz'));

    await expect(page.locator('text=/no ranked prospects/i')).toBeVisible();

    await actor.attemptsTo(ClearAllFilters.now());

    const searchInput = page.locator('#search');
    await expect(searchInput).toHaveValue('');

    const rows = page.locator('tbody tr');
    await expect(rows.first()).toBeVisible();
  });

  test('should navigate to player detail when clicking a row', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    const firstRow = page.locator('tbody tr[role="button"]').first();
    await firstRow.click();

    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/players\/[a-f0-9-]+/);
  });

  test('should display position group dropdown', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    const groupSelect = page.locator('select#group');
    await expect(groupSelect).toBeVisible();

    const options = groupSelect.locator('option');
    await expect(options).toHaveCount(4);
    await expect(options.nth(0)).toHaveText('All Groups');
    await expect(options.nth(1)).toHaveText('Offense');
    await expect(options.nth(2)).toHaveText('Defense');
    await expect(options.nth(3)).toHaveText('Special Teams');
  });

  test('should filter prospects by position group', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    await actor.attemptsTo(FilterByPositionGroup.named('Offense'));

    await expect(page.locator('text=/Group: offense/i')).toBeVisible();

    const offensePositions = ['QB', 'RB', 'WR', 'TE', 'OT', 'OG', 'C'];
    const rows = page.locator('tbody tr');
    const rowCount = await rows.count();

    if (rowCount > 0) {
      for (let i = 0; i < Math.min(rowCount, 5); i++) {
        const posText = await rows.nth(i).locator('td').nth(2).textContent();
        expect(offensePositions).toContain(posText?.trim());
      }
    }
  });

  test('should reset position when group changes', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    await actor.attemptsTo(SelectPosition.named('QB'));
    await expect(page.locator('select#position')).toHaveValue('QB');

    await actor.attemptsTo(FilterByPositionGroup.named('Defense'));

    await expect(page.locator('select#position')).toHaveValue('all');
  });

  test('should scope position dropdown to selected group', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    await actor.attemptsTo(FilterByPositionGroup.named('Offense'));

    // Position dropdown should show All + 7 offense positions = 8 options
    const positionOptions = page.locator('select#position option');
    await expect(positionOptions).toHaveCount(8);
  });

  test('should filter by group and specific position together', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    await actor.attemptsTo(FilterByPositionGroup.named('Defense'));
    await actor.attemptsTo(SelectPosition.named('CB'));

    const rows = page.locator('tbody tr');
    const rowCount = await rows.count();

    if (rowCount > 0) {
      for (let i = 0; i < Math.min(rowCount, 5); i++) {
        const posCell = rows.nth(i).locator('td').nth(2);
        await expect(posCell).toContainText('CB');
      }
    }
  });

  test('should clear group filter with Clear all', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForSelector('table', { timeout: 10000 });

    await actor.attemptsTo(FilterByPositionGroup.named('Offense'));
    await expect(page.locator('text=/Group: offense/i')).toBeVisible();

    await actor.attemptsTo(ClearAllFilters.now());

    await expect(page.locator('select#group')).toHaveValue('all');
    await expect(page.locator('text=/Group: offense/i')).not.toBeVisible();
  });

  test('should show unranked player count with link to /players', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toProspects());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');
    await page.waitForSelector('table', { timeout: 10000 });

    const unrankedLink = page.locator('a[href="/players"]', { hasText: /view all players/i });
    await expect(unrankedLink).toBeVisible({ timeout: 10000 });
  });
});

test.describe('Prospects Navigation', () => {
  test('should have Prospects link in desktop navigation', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toHomePage());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const navLink = page.locator('nav a[href="/prospects"]');
    await expect(navLink).toBeVisible();
    await expect(navLink).toContainText('Prospects');
  });

  test('should navigate to prospects page via nav link', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toHomePage());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.locator('nav a[href="/prospects"]').click();
    await page.waitForLoadState('networkidle');

    const url = await actor.asks(CurrentUrl.value());
    expect(url).toContain('/prospects');

    const heading = await actor.asks(PageHeading.text());
    expect(heading).toMatch(/prospect rankings/i);
  });
});
