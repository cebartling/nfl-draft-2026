import type { Actor, Question } from '../actor.js';
import { QueryDatabase } from '../abilities/query-database.js';

class DraftStatusQuestion implements Question<string | null> {
  constructor(private readonly draftId: string) {}

  async answeredBy(actor: Actor): Promise<string | null> {
    const db = actor.abilityTo(QueryDatabase);
    const row = await db.queryOne<{ status: string }>(
      'SELECT status FROM drafts WHERE id = $1',
      [this.draftId],
    );
    return row?.status ?? null;
  }
}

class DraftPickCountQuestion implements Question<number> {
  constructor(private readonly draftId: string) {}

  async answeredBy(actor: Actor): Promise<number> {
    const db = actor.abilityTo(QueryDatabase);
    return db.count('draft_picks', 'draft_id = $1', [this.draftId]);
  }
}

class MadePickCountQuestion implements Question<number> {
  constructor(private readonly draftId: string) {}

  async answeredBy(actor: Actor): Promise<number> {
    const db = actor.abilityTo(QueryDatabase);
    return db.count('draft_picks', 'draft_id = $1 AND player_id IS NOT NULL', [this.draftId]);
  }
}

class SessionStatusQuestion implements Question<string | null> {
  constructor(private readonly draftId: string) {}

  async answeredBy(actor: Actor): Promise<string | null> {
    const db = actor.abilityTo(QueryDatabase);
    const row = await db.queryOne<{ status: string }>(
      'SELECT status FROM draft_sessions WHERE draft_id = $1 ORDER BY created_at DESC LIMIT 1',
      [this.draftId],
    );
    return row?.status ?? null;
  }
}

export const DraftStatus = {
  inDatabaseFor(draftId: string): Question<string | null> {
    return new DraftStatusQuestion(draftId);
  },
};

export const DraftPickCount = {
  inDatabaseFor(draftId: string): Question<number> {
    return new DraftPickCountQuestion(draftId);
  },
};

export const MadePickCount = {
  inDatabaseFor(draftId: string): Question<number> {
    return new MadePickCountQuestion(draftId);
  },
};

export const SessionStatus = {
  inDatabaseForDraft(draftId: string): Question<string | null> {
    return new SessionStatusQuestion(draftId);
  },
};

interface DraftDetailsResult {
  name: string;
  status: string;
  rounds: number;
  is_realistic: boolean;
  picks_per_round: number | null;
}

interface DraftDetailsRow {
  name: string;
  status: string;
  rounds: number;
  picks_per_round: number | null;
}

class DraftDetailsQuestion implements Question<DraftDetailsResult | null> {
  constructor(private readonly draftId: string) {}

  async answeredBy(actor: Actor): Promise<DraftDetailsResult | null> {
    const db = actor.abilityTo(QueryDatabase);
    const row = await db.queryOne<DraftDetailsRow>(
      'SELECT name, status, rounds, picks_per_round FROM drafts WHERE id = $1',
      [this.draftId],
    );
    if (!row) return null;
    return {
      ...row,
      is_realistic: row.picks_per_round === null,
    };
  }
}

export const DraftDetails = {
  inDatabaseFor(draftId: string): Question<DraftDetailsResult | null> {
    return new DraftDetailsQuestion(draftId);
  },
};
