import type { Actor, Question } from '../actor.js';
import { QueryDatabase } from '../abilities/query-database.js';

interface DbTeam {
  id: string;
  name: string;
  abbreviation: string;
  city: string;
  conference: string;
  division: string;
}

class TeamCountQuestion implements Question<number> {
  async answeredBy(actor: Actor): Promise<number> {
    const db = actor.abilityTo(QueryDatabase);
    return db.count('teams');
  }
}

class TeamDetailsQuestion implements Question<DbTeam | null> {
  constructor(private readonly abbreviation: string) {}

  async answeredBy(actor: Actor): Promise<DbTeam | null> {
    const db = actor.abilityTo(QueryDatabase);
    return db.queryOne<DbTeam>('SELECT * FROM teams WHERE abbreviation = $1', [
      this.abbreviation,
    ]);
  }
}

export const TeamCount = {
  inDatabase(): Question<number> {
    return new TeamCountQuestion();
  },
};

export const TeamDetails = {
  inDatabaseFor(abbreviation: string): Question<DbTeam | null> {
    return new TeamDetailsQuestion(abbreviation);
  },
};
