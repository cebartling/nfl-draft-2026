import { test, expect } from '../src/fixtures/test-fixture.js';
import {
  CreateDraftViaApi,
  StartSession,
} from '../src/screenplay/tasks/draft-tasks.js';
import { RunAutoPickForSession } from '../src/screenplay/tasks/session-tasks.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import {
  WaitForWebSocketConnection,
  WaitForPickCardUpdate,
  WaitForAllPicksFilled,
} from '../src/screenplay/tasks/session-page-tasks.js';
import {
  PickCardHasPlayer,
  DraftBoardFilledPickCount,
  CurrentPickHighlight,
  PickNotificationCount,
  SessionStatusText,
} from '../src/screenplay/questions/session-page-questions.js';
import {
  DraftStatus,
  MadePickCount,
  SessionStatus,
} from '../src/screenplay/questions/draft-questions.js';
import { BrowseTheWeb } from '../src/screenplay/abilities/browse-the-web.js';
import { CallApi } from '../src/screenplay/abilities/call-api.js';
import { cleanupTestDrafts } from '../src/db/cleanup.js';

/**
 * Helper: creates a 1-round draft via API, starts a session with no controlled teams,
 * navigates to the session page, and waits for WebSocket connection + draft board render.
 *
 * Returns the draft and session objects for further use.
 */
async function setupDraftSession(actor: InstanceType<typeof import('../src/screenplay/actor.js').Actor>) {
  // Create draft via API (1 round = 32 picks)
  const createTask = CreateDraftViaApi.named('E2E Auto-Pick Notifications Draft').withRounds(1);
  await actor.attemptsTo(createTask);
  const draft = createTask.response!;

  // Start session via API (no controlled teams — all AI)
  const startTask = StartSession.forDraft(draft.id).withTimePer(30);
  await actor.attemptsTo(startTask);
  const session = startTask.sessionResponse!;

  // Navigate to session page
  await actor.attemptsTo(Navigate.to(`/sessions/${session.id}`));

  // Wait for WebSocket to connect and draft board to render
  const page = actor.abilityTo(BrowseTheWeb).getPage();
  // Wait for draft board to be visible (indicates page has loaded)
  await page.locator('[data-testid="draft-board"], [data-pick="1"]').first().waitFor({
    state: 'visible',
    timeout: 15_000,
  });

  return { draft, session };
}

test.describe('Auto-Pick Real-Time Notifications', () => {
  test.afterAll(async () => {
    await cleanupTestDrafts();
  });

  test('pick cards update with player names in real-time during auto-pick', async ({
    actor,
  }) => {
    const { draft, session } = await setupDraftSession(actor);

    // Fire auto-pick-run API call without awaiting — let it run in background
    // so we can observe real-time WS updates arriving before the HTTP response
    const api = actor.abilityTo(CallApi);
    const autoPickPromise = api.post(`/api/v1/sessions/${session.id}/auto-pick-run`);

    // Wait for the first pick card to show a player name
    // This proves WebSocket updates arrive before the full HTTP response
    await actor.attemptsTo(WaitForPickCardUpdate.forPick(1).withTimeout(30_000));

    // Verify pick 1 has a player
    const pick1HasPlayer = await actor.asks(PickCardHasPlayer.forPick(1));
    expect(pick1HasPlayer).toBe(true);

    // Wait for pick 5 to also be filled (proves multiple WS updates are arriving)
    await actor.attemptsTo(WaitForPickCardUpdate.forPick(5).withTimeout(30_000));

    // Now await the auto-pick completion
    const result = await autoPickPromise;
    expect(result.ok).toBe(true);

    // Wait for all 32 pick cards to have player names
    await actor.attemptsTo(WaitForAllPicksFilled.expecting(32).withTimeout(60_000));

    // Verify all 32 pick cards show player names
    const filledCount = await actor.asks(DraftBoardFilledPickCount.onPage());
    expect(filledCount).toBe(32);

    // Verify DB: 32 picks made
    const madeCount = await actor.asks(MadePickCount.inDatabaseFor(draft.id));
    expect(madeCount).toBe(32);
  });

  test('current pick highlight advances during auto-pick', async ({ actor }) => {
    const { draft, session } = await setupDraftSession(actor);

    // Verify initial highlight is on pick 1
    const initialHighlight = await actor.asks(CurrentPickHighlight.onPage());
    // Highlight may be 1 or null if not yet rendered with data-current attribute
    if (initialHighlight !== null) {
      expect(initialHighlight).toBe(1);
    }

    // Fire auto-pick (don't await)
    const api = actor.abilityTo(CallApi);
    const autoPickPromise = api.post(`/api/v1/sessions/${session.id}/auto-pick-run`);

    // Wait until highlighted pick number advances past 1
    // This proves the frontend is advancing the current pick indicator via WS
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForFunction(
      () => {
        const el = document.querySelector('[data-pick][data-current="true"]');
        if (!el) return false;
        const pick = parseInt(el.getAttribute('data-pick') || '0', 10);
        return pick > 1;
      },
      { timeout: 30_000 },
    );

    // Await completion
    await autoPickPromise;

    // After completion, verify draft is completed in DB
    const draftStatus = await actor.asks(DraftStatus.inDatabaseFor(draft.id));
    expect(draftStatus).toBe('Completed');
  });

  test('pick notifications appear during auto-pick', async ({ actor }) => {
    const { draft, session } = await setupDraftSession(actor);

    // Fire auto-pick (don't await)
    const api = actor.abilityTo(CallApi);
    const autoPickPromise = api.post(`/api/v1/sessions/${session.id}/auto-pick-run`);

    // Wait for at least one pick notification to appear
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page
      .locator('[data-testid="pick-notification"]')
      .first()
      .waitFor({ state: 'visible', timeout: 30_000 });

    // Verify at least one notification contains a player name and team text
    const firstNotification = page.locator('[data-testid="pick-notification"]').first();
    const notificationText = await firstNotification.textContent();
    expect(notificationText).toBeTruthy();
    // Notification should contain at least some text (player/team info)
    expect(notificationText!.length).toBeGreaterThan(0);

    // Await completion
    await autoPickPromise;

    // Wait for all notifications to arrive
    await page.waitForFunction(
      () => {
        const notifications = document.querySelectorAll(
          '[data-testid="pick-notification"]',
        );
        return notifications.length >= 32;
      },
      { timeout: 60_000 },
    );

    // Verify notification count matches total picks
    const notificationCount = await actor.asks(PickNotificationCount.onPage());
    expect(notificationCount).toBe(32);
  });

  test('session status updates to Completed via WebSocket', async ({ actor }) => {
    const { draft, session } = await setupDraftSession(actor);

    // Verify initial status shows "In Progress" or "InProgress"
    const initialStatus = await actor.asks(SessionStatusText.onPage());
    expect(initialStatus.toLowerCase()).toContain('progress');

    // Fire auto-pick and await completion
    const autoPickTask = RunAutoPickForSession.once(session.id);
    await actor.attemptsTo(autoPickTask);

    // Wait for session status to change to "Completed" via WebSocket (no page reload)
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.waitForFunction(
      () => {
        const el = document.querySelector('[data-testid="session-status"]');
        return el?.textContent?.toLowerCase().includes('completed');
      },
      { timeout: 15_000 },
    );

    const finalStatus = await actor.asks(SessionStatusText.onPage());
    expect(finalStatus.toLowerCase()).toContain('completed');

    // Verify DB: draft and session are both Completed
    const draftStatus = await actor.asks(DraftStatus.inDatabaseFor(draft.id));
    expect(draftStatus).toBe('Completed');

    const sessionStatus = await actor.asks(SessionStatus.inDatabaseForDraft(draft.id));
    expect(sessionStatus).toBe('Completed');
  });

  test('draft board pick cards show player position and college', async ({ actor }) => {
    const { draft, session } = await setupDraftSession(actor);

    // Run auto-pick to completion
    const autoPickTask = RunAutoPickForSession.once(session.id);
    await actor.attemptsTo(autoPickTask);

    // Wait for all 32 picks to be filled on the board
    await actor.attemptsTo(WaitForAllPicksFilled.expecting(32).withTimeout(60_000));

    // Verify at least one pick card shows position text (e.g., "QB", "WR", "OT")
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const positionElements = page.locator('[data-pick] [data-testid="pick-player-position"]');
    const positionCount = await positionElements.count();
    expect(positionCount).toBeGreaterThan(0);

    // Verify at least one position element has recognizable text
    const firstPositionText = await positionElements.first().textContent();
    expect(firstPositionText).toBeTruthy();
    expect(firstPositionText!.trim().length).toBeGreaterThan(0);

    // Verify at least one pick card shows college text
    const collegeElements = page.locator('[data-pick] [data-testid="pick-player-college"]');
    const collegeCount = await collegeElements.count();
    expect(collegeCount).toBeGreaterThan(0);

    const firstCollegeText = await collegeElements.first().textContent();
    expect(firstCollegeText).toBeTruthy();
    expect(firstCollegeText!.trim().length).toBeGreaterThan(0);
  });
});
