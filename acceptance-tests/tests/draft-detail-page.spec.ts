import { test, expect } from '../src/fixtures/test-fixture.js';
import { CreateDraftViaApi, StartSession } from '../src/screenplay/tasks/draft-tasks.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import {
  ClickAutoPickAllTeams,
  ClickCancel,
  SelectTeam,
} from '../src/screenplay/tasks/draft-page-tasks.js';
import {
  TeamSelectorVisible,
  AutoPickButtonVisible,
  StartButtonText,
  CancelButtonVisible,
  SelectedTeamCount,
  DraftProgressVisible,
  DraftBoardVisible,
  DraftStatisticsVisible,
} from '../src/screenplay/questions/draft-detail-questions.js';
import { CurrentUrl } from '../src/screenplay/questions/web-questions.js';
import { SessionStatus } from '../src/screenplay/questions/draft-questions.js';
import { cleanupTestDrafts } from '../src/db/cleanup.js';

test.describe('Draft Detail Page', () => {
  test.afterAll(async () => {
    await cleanupTestDrafts();
  });

  test('NotStarted draft shows unified panel with team selector', async ({ actor }) => {
    // Create draft via API
    const createTask = CreateDraftViaApi.named('E2E Detail Panel Test').withRounds(1);
    await actor.attemptsTo(createTask);
    const draft = createTask.response!;

    // Navigate to draft detail page
    await actor.attemptsTo(Navigate.to(`/drafts/${draft.id}`));

    // Wait for the page to load and teams to be fetched
    const page = (await import('../src/screenplay/abilities/browse-the-web.js')).BrowseTheWeb;
    const actorPage = actor.abilityTo(page).getPage();
    await actorPage.waitForSelector('[data-testid="auto-pick-all"]', { timeout: 10000 });

    // Verify team selector is immediately visible
    const teamSelectorVisible = await actor.asks(TeamSelectorVisible.onPage());
    expect(teamSelectorVisible).toBe(true);

    // Verify all three action buttons are present
    const autoPickVisible = await actor.asks(AutoPickButtonVisible.onPage());
    expect(autoPickVisible).toBe(true);

    const startText = await actor.asks(StartButtonText.onPage());
    expect(startText).toContain('Start with');

    const cancelVisible = await actor.asks(CancelButtonVisible.onPage());
    expect(cancelVisible).toBe(true);
  });

  test('can select teams and see count update', async ({ actor }) => {
    // Create draft via API
    const createTask = CreateDraftViaApi.named('E2E Team Selection Test').withRounds(1);
    await actor.attemptsTo(createTask);
    const draft = createTask.response!;

    // Navigate to draft detail page
    await actor.attemptsTo(Navigate.to(`/drafts/${draft.id}`));

    // Wait for the page to load
    const page = (await import('../src/screenplay/abilities/browse-the-web.js')).BrowseTheWeb;
    const actorPage = actor.abilityTo(page).getPage();
    await actorPage.waitForSelector('[data-testid="auto-pick-all"]', { timeout: 10000 });

    // Verify initial state shows 0 teams selected
    const startText = await actor.asks(StartButtonText.onPage());
    expect(startText).toBe('Start with 0 Teams');

    // Select a team (expand a division and click a team)
    await actor.attemptsTo(SelectTeam.byName('Kansas City Chiefs'));

    // Verify the button text updated
    const updatedText = await actor.asks(StartButtonText.onPage());
    expect(updatedText).toBe('Start with 1 Team');

    // Verify selected team count
    const count = await actor.asks(SelectedTeamCount.onPage());
    expect(count).toBe(1);
  });

  test('auto-pick all teams creates session and redirects', async ({ actor }) => {
    // Create draft via API
    const createTask = CreateDraftViaApi.named('E2E Auto-Pick Test').withRounds(1);
    await actor.attemptsTo(createTask);
    const draft = createTask.response!;

    // Navigate to draft detail page
    await actor.attemptsTo(Navigate.to(`/drafts/${draft.id}`));

    // Wait for the page to load
    const page = (await import('../src/screenplay/abilities/browse-the-web.js')).BrowseTheWeb;
    const actorPage = actor.abilityTo(page).getPage();
    await actorPage.waitForSelector('[data-testid="auto-pick-all"]', { timeout: 10000 });

    // Click "Auto-pick All Teams"
    await actor.attemptsTo(ClickAutoPickAllTeams.now());

    // Wait for navigation to session page
    await actorPage.waitForURL(/\/sessions\/[0-9a-f-]+/, { timeout: 15000 });

    // Verify we're on a session page
    const currentUrl = await actor.asks(CurrentUrl.value());
    expect(currentUrl).toMatch(/\/sessions\/[0-9a-f-]+/);

    // Verify session was created in DB
    const sessionStatus = await actor.asks(SessionStatus.inDatabaseForDraft(draft.id));
    expect(sessionStatus).not.toBeNull();
  });

  test('cancel navigates back to drafts list', async ({ actor }) => {
    // Create draft via API
    const createTask = CreateDraftViaApi.named('E2E Cancel Test').withRounds(1);
    await actor.attemptsTo(createTask);
    const draft = createTask.response!;

    // Navigate to draft detail page
    await actor.attemptsTo(Navigate.to(`/drafts/${draft.id}`));

    // Wait for the page to load
    const page = (await import('../src/screenplay/abilities/browse-the-web.js')).BrowseTheWeb;
    const actorPage = actor.abilityTo(page).getPage();
    await actorPage.waitForSelector('[data-testid="cancel-draft"]', { timeout: 10000 });

    // Click "Cancel"
    await actor.attemptsTo(ClickCancel.now());

    // Wait for navigation
    await actorPage.waitForURL(/\/drafts$/, { timeout: 10000 });

    // Verify URL is /drafts
    const currentUrl = await actor.asks(CurrentUrl.value());
    expect(currentUrl).toMatch(/\/drafts$/);
  });

  test('NotStarted draft hides draft progress, board, and statistics', async ({ actor }) => {
    // Create draft via API
    const createTask = CreateDraftViaApi.named('E2E Hidden Sections Test').withRounds(1);
    await actor.attemptsTo(createTask);
    const draft = createTask.response!;

    // Navigate to draft detail page
    await actor.attemptsTo(Navigate.to(`/drafts/${draft.id}`));

    // Wait for the page to load
    const page = (await import('../src/screenplay/abilities/browse-the-web.js')).BrowseTheWeb;
    const actorPage = actor.abilityTo(page).getPage();
    await actorPage.waitForSelector('[data-testid="auto-pick-all"]', { timeout: 10000 });

    // Verify draft progress is NOT visible
    const progressVisible = await actor.asks(DraftProgressVisible.onPage());
    expect(progressVisible).toBe(false);

    // Verify draft board is NOT visible
    const boardVisible = await actor.asks(DraftBoardVisible.onPage());
    expect(boardVisible).toBe(false);

    // Verify draft statistics is NOT visible
    const statsVisible = await actor.asks(DraftStatisticsVisible.onPage());
    expect(statsVisible).toBe(false);
  });

  test('InProgress draft shows draft progress, board, and statistics', async ({ actor }) => {
    // Create draft + session via API
    const createTask = CreateDraftViaApi.named('E2E Visible Sections Test').withRounds(1);
    await actor.attemptsTo(createTask);
    const draft = createTask.response!;

    // Start a session via API
    const startTask = StartSession.forDraft(draft.id).withTimePer(30);
    await actor.attemptsTo(startTask);

    // Navigate to draft detail page
    await actor.attemptsTo(Navigate.to(`/drafts/${draft.id}`));

    // Wait for the page to load
    const page = (await import('../src/screenplay/abilities/browse-the-web.js')).BrowseTheWeb;
    const actorPage = actor.abilityTo(page).getPage();
    await actorPage.waitForSelector('[data-testid="draft-board"]', { timeout: 10000 });

    // Verify draft progress is visible
    const progressVisible = await actor.asks(DraftProgressVisible.onPage());
    expect(progressVisible).toBe(true);

    // Verify draft board is visible
    const boardVisible = await actor.asks(DraftBoardVisible.onPage());
    expect(boardVisible).toBe(true);

    // Verify draft statistics is visible
    const statsVisible = await actor.asks(DraftStatisticsVisible.onPage());
    expect(statsVisible).toBe(true);
  });

  test('InProgress draft shows session info, not setup panel', async ({ actor }) => {
    // Create draft + session via API
    const createTask = CreateDraftViaApi.named('E2E InProgress Test').withRounds(1);
    await actor.attemptsTo(createTask);
    const draft = createTask.response!;

    // Start a session via API
    const startTask = StartSession.forDraft(draft.id).withTimePer(30);
    await actor.attemptsTo(startTask);

    // Navigate to draft detail page
    await actor.attemptsTo(Navigate.to(`/drafts/${draft.id}`));

    // Wait for the page to load
    const page = (await import('../src/screenplay/abilities/browse-the-web.js')).BrowseTheWeb;
    const actorPage = actor.abilityTo(page).getPage();
    await actorPage.waitForSelector('h1', { timeout: 10000 });

    // Verify team selector is NOT visible
    const teamSelectorVisible = await actor.asks(TeamSelectorVisible.onPage());
    expect(teamSelectorVisible).toBe(false);

    // Verify "Auto-pick All Teams" button is NOT visible
    const autoPickVisible = await actor.asks(AutoPickButtonVisible.onPage());
    expect(autoPickVisible).toBe(false);

    // Verify "Join Session" button IS visible
    const joinButton = actorPage.getByRole('button', { name: /join session/i });
    await expect(joinButton).toBeVisible();
  });
});
