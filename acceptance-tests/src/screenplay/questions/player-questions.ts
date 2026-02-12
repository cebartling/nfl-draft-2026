import type { Actor, Question } from '../actor.js';
import { QueryDatabase } from '../abilities/query-database.js';

class PlayerCountQuestion implements Question<number> {
  async answeredBy(actor: Actor): Promise<number> {
    const db = actor.abilityTo(QueryDatabase);
    return db.count('players');
  }
}

class PlayerDetailsQuestion implements Question<Record<string, unknown> | null> {
  constructor(private readonly playerId: string) {}

  async answeredBy(actor: Actor): Promise<Record<string, unknown> | null> {
    const db = actor.abilityTo(QueryDatabase);
    return db.queryOne('SELECT * FROM players WHERE id = $1', [this.playerId]);
  }
}

export const PlayerCount = {
  inDatabase(): Question<number> {
    return new PlayerCountQuestion();
  },
};

export const PlayerDetails = {
  inDatabaseFor(playerId: string): Question<Record<string, unknown> | null> {
    return new PlayerDetailsQuestion(playerId);
  },
};
