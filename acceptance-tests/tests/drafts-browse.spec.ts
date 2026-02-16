import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { PageHeading } from '../src/screenplay/questions/web-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';

test.describe('Drafts List Page', () => {
  test('loads with heading', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toDrafts());

    const heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('Drafts');
  });

  test('Create New Draft button is visible', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toDrafts());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await expect(page.getByRole('button', { name: /Create New Draft/i })).toBeVisible();
  });

  test('status filter buttons are visible', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toDrafts());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await expect(page.locator('button', { hasText: /^All/ })).toBeVisible();
    await expect(page.locator('button', { hasText: /^Pending/ })).toBeVisible();
    await expect(page.locator('button', { hasText: /^Active/ })).toBeVisible();
    await expect(page.locator('button', { hasText: /^Completed/ })).toBeVisible();
  });
});

test.describe('Create Draft Page', () => {
  test('shows name field and rounds slider', async ({ actor }) => {
    await actor.attemptsTo(Navigate.to('/drafts/new'));

    const page = actor.abilityTo(BrowseTheWeb).getPage();

    await expect(page.getByRole('heading', { name: /Create New Draft/i })).toBeVisible();
    await expect(page.getByLabel(/Draft Name/i)).toBeVisible();
    await expect(page.getByLabel(/Draft Name/i)).toHaveValue(/.+/);
    await expect(page.getByLabel(/Number of Rounds/i)).toBeVisible();
  });

  test('shows summary and action buttons', async ({ actor }) => {
    await actor.attemptsTo(Navigate.to('/drafts/new'));

    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // Summary shows Year, Rounds, Realistic
    await expect(page.getByText('2026', { exact: true })).toBeVisible();
    await expect(page.getByText('Rounds', { exact: true })).toBeVisible();
    await expect(page.getByText('Realistic')).toBeVisible();

    // Action buttons
    await expect(page.getByRole('button', { name: /Cancel/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Back to Drafts/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Create Draft/i })).toBeVisible();
  });
});
