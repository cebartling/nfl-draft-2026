import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { PageHeading } from '../src/screenplay/questions/web-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';

test.describe('Home Page', () => {
  test('displays hero section with heading and CTA', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toHomePage());

    const page = actor.abilityTo(BrowseTheWeb).getPage();

    const heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('NFL Draft Simulator 2026');

    await expect(
      page.getByRole('button', { name: /Create New Draft/i }),
    ).toBeVisible();
  });

  test('Create New Draft button navigates to /drafts/new', async ({
    actor,
  }) => {
    await actor.attemptsTo(Navigate.toHomePage());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await page.getByRole('button', { name: /Create New Draft/i }).click();
    await page.waitForURL(/\/drafts\/new/);

    expect(page.url()).toContain('/drafts/new');
  });

  test('displays Features section with four feature cards', async ({
    actor,
  }) => {
    await actor.attemptsTo(Navigate.toHomePage());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await expect(
      page.getByRole('heading', { name: 'Features' }),
    ).toBeVisible();
    await expect(
      page.getByRole('heading', { name: 'Real-time Updates' }),
    ).toBeVisible();
    await expect(
      page.getByRole('heading', { name: 'AI Decision Making' }),
    ).toBeVisible();
    await expect(
      page.getByRole('heading', { name: 'Comprehensive Scouting' }),
    ).toBeVisible();
    await expect(
      page.getByRole('heading', { name: 'Trade System' }),
    ).toBeVisible();
  });

  test('displays Recent Drafts section with View All link', async ({
    actor,
  }) => {
    await actor.attemptsTo(Navigate.toHomePage());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await expect(
      page.getByRole('heading', { name: 'Recent Drafts' }),
    ).toBeVisible();
    await expect(page.getByRole('link', { name: 'View All' })).toBeVisible();
  });

  test('View All link navigates to /drafts', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toHomePage());

    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForLoadState('networkidle');

    await page.getByRole('link', { name: 'View All' }).click();
    await page.waitForURL(/\/drafts/);

    expect(page.url()).toContain('/drafts');
  });
});
