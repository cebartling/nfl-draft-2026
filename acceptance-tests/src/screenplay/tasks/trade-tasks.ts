import type { Actor, Task } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';
import { CallApi } from '../abilities/call-api.js';

interface TradeResponse {
  id: string;
  session_id: string;
  from_team_id: string;
  to_team_id: string;
  status: string;
  from_team_value: number;
  to_team_value: number;
  value_difference: number;
}

interface TradeProposalResponse {
  trade: TradeResponse;
  from_team_picks: string[];
  to_team_picks: string[];
}

interface ProposeTradeParams {
  sessionId: string;
  fromTeamId: string;
  toTeamId: string;
  fromTeamPicks: string[];
  toTeamPicks: string[];
  chartType?: string;
}

class ProposeTradeViaApiTask implements Task {
  public response: TradeProposalResponse | null = null;

  constructor(private readonly params: ProposeTradeParams) {}

  async performAs(actor: Actor): Promise<void> {
    const api = actor.abilityTo(CallApi);
    const res = await api.post<TradeProposalResponse>('/api/v1/trades', {
      session_id: this.params.sessionId,
      from_team_id: this.params.fromTeamId,
      to_team_id: this.params.toTeamId,
      from_team_picks: this.params.fromTeamPicks,
      to_team_picks: this.params.toTeamPicks,
      chart_type: this.params.chartType ?? 'JimmyJohnson',
    });
    if (!res.ok) {
      throw new Error(
        `Failed to propose trade: ${res.status} ${JSON.stringify(res.data)}`,
      );
    }
    this.response = res.data;
  }
}

export const ProposeTradeViaApi = {
  with(params: ProposeTradeParams): ProposeTradeViaApiTask {
    return new ProposeTradeViaApiTask(params);
  },
};

class ClickTabTask implements Task {
  constructor(private readonly tabId: string) {}

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.locator(`#tab-${this.tabId}`).click();
  }
}

export const ClickTab = {
  withId(id: string): ClickTabTask {
    return new ClickTabTask(id);
  },
};
