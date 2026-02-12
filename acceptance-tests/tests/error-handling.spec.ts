import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';

const NONEXISTENT_UUID = '00000000-0000-0000-0000-000000000000';

test.describe('Error Handling', () => {
  test('navigating to a nonexistent draft shows error state', async ({ actor }) => {
    await actor.attemptsTo(Navigate.to(`/drafts/${NONEXISTENT_UUID}`));

    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // The app renders an h2 "Draft Not Found" for missing drafts
    await expect(page.getByRole('heading', { name: 'Draft Not Found' })).toBeVisible();
  });

  test('navigating to a nonexistent team shows error state', async ({ actor }) => {
    await actor.attemptsTo(Navigate.to(`/teams/${NONEXISTENT_UUID}`));

    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // The app renders an h2 "Team Not Found" for missing teams
    await expect(page.getByRole('heading', { name: 'Team Not Found' })).toBeVisible();
  });
});
