import type { Actor, Task } from '../actor.js';
import { CallApi } from '../abilities/call-api.js';

interface AutoPickRunResponse {
  session: {
    id: string;
    status: string;
    current_pick_number: number;
  };
  picks_made: Array<{
    id: string;
    player_id: string | null;
  }>;
}

class RunAutoPickForSessionTask implements Task {
  public result: AutoPickRunResponse | null = null;

  constructor(private readonly sessionId: string) {}

  async performAs(actor: Actor): Promise<void> {
    const api = actor.abilityTo(CallApi);
    const res = await api.post<AutoPickRunResponse>(
      `/api/v1/sessions/${this.sessionId}/auto-pick-run`,
    );
    if (!res.ok) throw new Error(`Auto-pick failed: ${res.status}`);
    this.result = res.data;
  }
}

class AdvancePickTask implements Task {
  constructor(private readonly sessionId: string) {}

  async performAs(actor: Actor): Promise<void> {
    const api = actor.abilityTo(CallApi);
    const res = await api.post(`/api/v1/sessions/${this.sessionId}/advance-pick`);
    if (!res.ok) throw new Error(`Advance pick failed: ${res.status}`);
  }
}

export const RunAutoPickForSession = {
  once(sessionId: string): RunAutoPickForSessionTask {
    return new RunAutoPickForSessionTask(sessionId);
  },
};

export const AdvancePick = {
  forSession(sessionId: string): AdvancePickTask {
    return new AdvancePickTask(sessionId);
  },
};
