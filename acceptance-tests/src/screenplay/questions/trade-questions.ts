import type { Actor, Question } from '../actor.js';
import { CallApi } from '../abilities/call-api.js';
import { QueryDatabase } from '../abilities/query-database.js';

interface TradeProposalResponse {
  trade: {
    id: string;
    session_id: string;
    status: string;
  };
  from_team_picks: string[];
  to_team_picks: string[];
}

/**
 * Count of trade rows in the pick_trades table for a given session.
 */
class TradeCountInDatabaseQuestion implements Question<number> {
  constructor(private readonly sessionId: string) {}

  async answeredBy(actor: Actor): Promise<number> {
    return actor
      .abilityTo(QueryDatabase)
      .count('pick_trades', 'session_id = $1', [this.sessionId]);
  }
}

export const TradeCountInDatabase = {
  forSession(sessionId: string): Question<number> {
    return new TradeCountInDatabaseQuestion(sessionId);
  },
};

/**
 * Result of GET /api/v1/sessions/{id}/trades.
 */
class SessionTradesFromApiQuestion implements Question<TradeProposalResponse[]> {
  constructor(private readonly sessionId: string) {}

  async answeredBy(actor: Actor): Promise<TradeProposalResponse[]> {
    const res = await actor
      .abilityTo(CallApi)
      .get<TradeProposalResponse[]>(`/api/v1/sessions/${this.sessionId}/trades`);
    if (!res.ok) {
      throw new Error(`Failed to get session trades: ${res.status}`);
    }
    return res.data;
  }
}

export const SessionTradesFromApi = {
  forSession(sessionId: string): Question<TradeProposalResponse[]> {
    return new SessionTradesFromApiQuestion(sessionId);
  },
};
