import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';
import {
  CreateDraftViaApi,
  StartSession,
} from '../src/screenplay/tasks/draft-tasks.js';

test.describe('Drafts Status Filtering', () => {
  test('Pending filter shows only NotStarted drafts', async ({ actor }) => {
    // Create a NotStarted draft via API
    const createTask = CreateDraftViaApi.named('E2E Filter Pending Draft');
    await actor.attemptsTo(createTask);
    expect(createTask.response).not.toBeNull();

    // Navigate to drafts page
    await actor.attemptsTo(Navigate.toDrafts());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    // Verify the draft appears in the All view
    await expect(
      page.getByText('E2E Filter Pending Draft').first(),
    ).toBeVisible();

    // Click Pending filter
    await page.locator('button', { hasText: /^Pending/ }).click();

    // The draft should still be visible (it's NotStarted = Pending)
    await expect(
      page.getByText('E2E Filter Pending Draft').first(),
    ).toBeVisible();
  });

  test('Active filter hides NotStarted drafts', async ({ actor }) => {
    // Create a NotStarted draft via API
    const createTask = CreateDraftViaApi.named('E2E Filter Active Test Draft');
    await actor.attemptsTo(createTask);
    expect(createTask.response).not.toBeNull();

    // Navigate to drafts page
    await actor.attemptsTo(Navigate.toDrafts());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    // Verify the draft appears in All view
    await expect(
      page.getByText('E2E Filter Active Test Draft').first(),
    ).toBeVisible();

    // Click Active filter
    await page.locator('button', { hasText: /^Active/ }).click();

    // The NotStarted draft should NOT be visible under Active filter
    await expect(
      page.getByText('E2E Filter Active Test Draft'),
    ).not.toBeVisible();
  });

  test('All filter shows drafts after switching from a specific filter', async ({
    actor,
  }) => {
    // Create a draft via API
    const createTask = CreateDraftViaApi.named('E2E Filter All Test Draft');
    await actor.attemptsTo(createTask);
    expect(createTask.response).not.toBeNull();

    // Navigate to drafts page
    await actor.attemptsTo(Navigate.toDrafts());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    // Switch to Completed filter (should hide the NotStarted draft)
    await page.locator('button', { hasText: /^Completed/ }).click();
    await expect(
      page.getByText('E2E Filter All Test Draft'),
    ).not.toBeVisible();

    // Switch back to All
    await page.locator('button', { hasText: /^All/ }).click();

    // Draft should be visible again
    await expect(
      page.getByText('E2E Filter All Test Draft').first(),
    ).toBeVisible();
  });
});
