import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { CreateDraft } from '../src/screenplay/tasks/draft-tasks.js';
import { CurrentUrl, PageHeading } from '../src/screenplay/questions/web-questions.js';
import { DraftPickCount, DraftDetails } from '../src/screenplay/questions/draft-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';
import { cleanupTestDrafts } from '../src/db/cleanup.js';

test.describe('Draft Lifecycle', () => {
  test.afterAll(async () => {
    await cleanupTestDrafts();
  });

  test('full draft lifecycle: create, verify detail page and DB state', async ({
    actor,
  }) => {
    // Navigate to create draft page
    await actor.attemptsTo(Navigate.to('/drafts/new'));

    // Create a draft through the UI
    const createTask = CreateDraft.named('E2E Lifecycle Test Draft').withRounds(1);
    await actor.attemptsTo(createTask);

    // Extract draft ID from URL
    const url = await actor.asks(CurrentUrl.value());
    const match = url.match(/\/drafts\/([0-9a-f-]+)/);
    expect(match).not.toBeNull();
    const draftId = match![1];

    // Verify the detail page heading shows the draft name
    const heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('E2E Lifecycle Test Draft');

    // Verify the status badge shows NotStarted on the detail page
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await expect(page.getByText('NotStarted')).toBeVisible();

    // The create draft page auto-initializes picks: 1 round × 32 teams = 32 picks
    const pickCount = await actor.asks(DraftPickCount.inDatabaseFor(draftId));
    expect(pickCount).toBe(32);

    // Verify draft details in database: is_realistic should be true (UI doesn't send picks_per_round)
    const details = await actor.asks(DraftDetails.inDatabaseFor(draftId));
    expect(details).not.toBeNull();
    expect(details!.name).toBe('E2E Lifecycle Test Draft');
    expect(details!.status).toBe('NotStarted');
    expect(details!.is_realistic).toBe(true);
    expect(details!.picks_per_round).toBeNull();
    expect(details!.rounds).toBe(1);
  });

  test('created draft appears on drafts list page', async ({ actor }) => {
    // Create a draft
    await actor.attemptsTo(Navigate.to('/drafts/new'));
    const createTask = CreateDraft.named('E2E List Test Draft').withRounds(1);
    await actor.attemptsTo(createTask);

    // Navigate to drafts list
    await actor.attemptsTo(Navigate.toDrafts());

    // Verify the draft appears in the list
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await expect(page.getByText('E2E List Test Draft')).toBeVisible();
  });

  test('multi-round draft: 3 rounds with correct pick count', async ({ actor }) => {
    // Navigate to create draft page
    await actor.attemptsTo(Navigate.to('/drafts/new'));

    // Create a 3-round draft through the UI
    const createTask = CreateDraft.named('E2E Multi-Round Draft').withRounds(3);
    await actor.attemptsTo(createTask);

    // Extract draft ID from URL
    const url = await actor.asks(CurrentUrl.value());
    const match = url.match(/\/drafts\/([0-9a-f-]+)/);
    expect(match).not.toBeNull();
    const draftId = match![1];

    // Verify the detail page heading shows the draft name
    const heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('E2E Multi-Round Draft');

    // Verify draft details in database
    const details = await actor.asks(DraftDetails.inDatabaseFor(draftId));
    expect(details).not.toBeNull();
    expect(details!.name).toBe('E2E Multi-Round Draft');
    expect(details!.rounds).toBe(3);
    expect(details!.is_realistic).toBe(true);
    expect(details!.picks_per_round).toBeNull();

    // Realistic 3-round draft: Round 1 = 32, Round 2 = 32, Round 3 = 36 (compensatory) = 100
    const pickCount = await actor.asks(DraftPickCount.inDatabaseFor(draftId));
    expect(pickCount).toBe(100);
  });
});
