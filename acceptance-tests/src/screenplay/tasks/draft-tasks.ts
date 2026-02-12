import type { Actor, Task } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';
import { CallApi } from '../abilities/call-api.js';

interface DraftResponse {
  id: string;
  name: string;
  year: number;
  status: string;
  rounds: number;
  picks_per_round: number | null;
  total_picks: number | null;
  is_realistic: boolean;
}

interface DraftPickResponse {
  id: string;
  draft_id: string;
  round: number;
  pick_number: number;
  overall_pick: number;
  team_id: string;
  player_id: string | null;
}

interface SessionResponse {
  id: string;
  draft_id: string;
  status: string;
  current_pick_number: number;
  time_per_pick_seconds: number;
  auto_pick_enabled: boolean;
}

class CreateDraftTask implements Task {
  private draftName: string;
  private rounds: number = 1;
  public createdDraftId: string | null = null;

  constructor(name: string) {
    this.draftName = name;
  }

  withRounds(n: number): this {
    this.rounds = n;
    return this;
  }

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // Fill in the draft name
    const nameInput = page.locator('input[type="text"]');
    await nameInput.clear();
    await nameInput.fill(this.draftName);

    // Set rounds via the range slider
    const slider = page.locator('input[type="range"]');
    await slider.fill(String(this.rounds));

    // Click the create button
    await page.getByRole('button', { name: /create draft/i }).click();

    // Wait for navigation to the draft detail page
    await page.waitForURL(/\/drafts\/[0-9a-f-]+/);

    // Extract the draft ID from the URL
    const url = page.url();
    const match = url.match(/\/drafts\/([0-9a-f-]+)/);
    this.createdDraftId = match ? match[1] : null;
  }
}

class CreateDraftViaApiTask implements Task {
  private draftName: string;
  private rounds: number = 1;
  public response: DraftResponse | null = null;

  constructor(name: string) {
    this.draftName = name;
  }

  withRounds(n: number): this {
    this.rounds = n;
    return this;
  }

  async performAs(actor: Actor): Promise<void> {
    const api = actor.abilityTo(CallApi);

    // Create draft
    const createRes = await api.post<DraftResponse>('/api/v1/drafts', {
      name: this.draftName,
      year: 2026,
      rounds: this.rounds,
      picks_per_round: 32,
    });
    if (!createRes.ok) throw new Error(`Failed to create draft: ${createRes.status}`);
    this.response = createRes.data;

    // Initialize picks
    const initRes = await api.post<DraftPickResponse[]>(
      `/api/v1/drafts/${this.response.id}/initialize`,
    );
    if (!initRes.ok) throw new Error(`Failed to initialize picks: ${initRes.status}`);
  }
}

class InitializePicksTask implements Task {
  constructor(private readonly draftId: string) {}

  async performAs(actor: Actor): Promise<void> {
    const api = actor.abilityTo(CallApi);
    const res = await api.post(`/api/v1/drafts/${this.draftId}/initialize`);
    if (!res.ok) throw new Error(`Failed to initialize picks: ${res.status}`);
  }
}

class StartSessionTask implements Task {
  private draftId: string;
  private timePerPick: number = 30;
  public sessionResponse: SessionResponse | null = null;

  constructor(draftId: string) {
    this.draftId = draftId;
  }

  withTimePer(seconds: number): this {
    this.timePerPick = seconds;
    return this;
  }

  async performAs(actor: Actor): Promise<void> {
    const api = actor.abilityTo(CallApi);

    // Start the draft
    const startRes = await api.post<DraftResponse>(`/api/v1/drafts/${this.draftId}/start`);
    if (!startRes.ok) throw new Error(`Failed to start draft: ${startRes.status}`);

    // Create a session
    const sessionRes = await api.post<SessionResponse>('/api/v1/sessions', {
      draft_id: this.draftId,
      time_per_pick_seconds: this.timePerPick,
      auto_pick_enabled: true,
      chart_type: 'JimmyJohnson',
      controlled_team_ids: [],
    });
    if (!sessionRes.ok) throw new Error(`Failed to create session: ${sessionRes.status}`);
    this.sessionResponse = sessionRes.data;

    // Start the session
    const startSessionRes = await api.post<SessionResponse>(
      `/api/v1/sessions/${this.sessionResponse.id}/start`,
    );
    if (!startSessionRes.ok) throw new Error(`Failed to start session: ${startSessionRes.status}`);
    this.sessionResponse = startSessionRes.data;
  }
}

export const CreateDraft = {
  named(name: string): CreateDraftTask {
    return new CreateDraftTask(name);
  },
};

export const CreateDraftViaApi = {
  named(name: string): CreateDraftViaApiTask {
    return new CreateDraftViaApiTask(name);
  },
};

export const InitializePicks = {
  forDraft(draftId: string): InitializePicksTask {
    return new InitializePicksTask(draftId);
  },
};

export const StartSession = {
  forDraft(draftId: string): StartSessionTask {
    return new StartSessionTask(draftId);
  },
};
