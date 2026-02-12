import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { CreateDraft } from '../src/screenplay/tasks/draft-tasks.js';
import { CurrentUrl } from '../src/screenplay/questions/web-questions.js';
import { DraftStatus, DraftPickCount } from '../src/screenplay/questions/draft-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';
import { cleanupTestDrafts } from '../src/db/cleanup.js';

test.describe('Draft Lifecycle', () => {
  test.afterAll(async () => {
    await cleanupTestDrafts();
  });

  test('full draft lifecycle: create, verify DB state, check picks initialized', async ({
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

    // Verify draft was created in database with correct status
    const status = await actor.asks(DraftStatus.inDatabaseFor(draftId));
    expect(status).toBe('NotStarted');

    // The create draft page auto-initializes picks, so verify pick count
    const pickCount = await actor.asks(DraftPickCount.inDatabaseFor(draftId));
    expect(pickCount).toBeGreaterThan(0);
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
});
