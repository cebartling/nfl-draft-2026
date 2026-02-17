import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { CurrentUrl } from '../src/screenplay/questions/web-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';
import { ExpandDivision } from '../src/screenplay/tasks/team-page-tasks.js';

test.describe('Team Detail Page', () => {
  test('displays conference and division info', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await actor.attemptsTo(ExpandDivision.named('AFC East'));

    await page.getByText('Buffalo Bills').first().click();
    await page.waitForURL(/\/teams\/[0-9a-f-]+/);

    await expect(page.locator('text=/AFC/i').first()).toBeVisible({ timeout: 5000 });
  });

  test('back navigation returns to teams list', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toTeams());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await actor.attemptsTo(ExpandDivision.named('AFC East'));

    await page.getByText('Buffalo Bills').first().click();
    await page.waitForURL(/\/teams\/[0-9a-f-]+/);

    await page.goBack();
    await page.waitForURL(/\/teams/);

    const url = await actor.asks(CurrentUrl.value());
    expect(url).toMatch(/\/teams/);
  });
});
