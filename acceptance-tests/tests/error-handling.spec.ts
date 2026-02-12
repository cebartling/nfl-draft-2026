import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';

const NONEXISTENT_UUID = '00000000-0000-0000-0000-000000000000';

test.describe('Error Handling', () => {
  test('navigating to a nonexistent draft shows error state', async ({ actor }) => {
    await actor.attemptsTo(Navigate.to(`/drafts/${NONEXISTENT_UUID}`));

    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // The page should show some error indication (error message, not found, etc.)
    // Wait for content to load
    await page.waitForLoadState('networkidle');

    // Look for error indicators - the app may show "not found", "error", or redirect
    const content = await page.textContent('body');
    const hasError =
      content?.toLowerCase().includes('not found') ||
      content?.toLowerCase().includes('error') ||
      content?.toLowerCase().includes('does not exist');
    expect(hasError).toBe(true);
  });

  test('navigating to a nonexistent team shows error state', async ({ actor }) => {
    await actor.attemptsTo(Navigate.to(`/teams/${NONEXISTENT_UUID}`));

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    const content = await page.textContent('body');
    const hasError =
      content?.toLowerCase().includes('not found') ||
      content?.toLowerCase().includes('error') ||
      content?.toLowerCase().includes('does not exist');
    expect(hasError).toBe(true);
  });
});
