import { test, expect } from '../src/fixtures/test-fixture.js';
import {
  CreateDraftViaApi,
  StartSession,
} from '../src/screenplay/tasks/draft-tasks.js';
import { RunAutoPickForSession } from '../src/screenplay/tasks/session-tasks.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import {
  DraftStatus,
  MadePickCount,
  SessionStatus,
} from '../src/screenplay/questions/draft-questions.js';
import { cleanupTestDrafts } from '../src/db/cleanup.js';

test.describe('Draft Session', () => {
  test.afterAll(async () => {
    await cleanupTestDrafts();
  });

  test('create session via API, navigate to it, run auto-pick', async ({ actor }) => {
    // Setup draft via API (fast, no browser needed)
    const createTask = CreateDraftViaApi.named('E2E Session Test Draft').withRounds(1);
    await actor.attemptsTo(createTask);

    const draft = createTask.response!;
    expect(draft.id).toMatch(/^[0-9a-f-]{36}$/);
    expect(draft.name).toBe('E2E Session Test Draft');
    expect(draft.status).toBe('NotStarted');
    expect(draft.rounds).toBe(1);

    // Start session via API
    const startTask = StartSession.forDraft(draft.id).withTimePer(30);
    await actor.attemptsTo(startTask);

    const session = startTask.sessionResponse!;
    expect(session.id).toMatch(/^[0-9a-f-]{36}$/);
    expect(session.draft_id).toBe(draft.id);
    expect(session.status).toBe('InProgress');

    // Verify session is in progress in DB
    const sessionStatus = await actor.asks(SessionStatus.inDatabaseForDraft(draft.id));
    expect(sessionStatus).toBe('InProgress');

    // Verify draft status changed
    const draftStatus = await actor.asks(DraftStatus.inDatabaseFor(draft.id));
    expect(draftStatus).toBe('InProgress');

    // Navigate to session page in browser
    await actor.attemptsTo(Navigate.to(`/sessions/${session.id}`));

    // Run auto-pick once via API
    const autoPickTask = RunAutoPickForSession.once(session.id);
    await actor.attemptsTo(autoPickTask);

    // Verify picks were made in DB
    const madeCount = await actor.asks(MadePickCount.inDatabaseFor(draft.id));
    expect(madeCount).toBeGreaterThan(0);
  });
});
