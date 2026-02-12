import type pg from 'pg';
import type { Ability } from '../actor.js';

export class QueryDatabase implements Ability {
  private constructor(private readonly pool: pg.Pool) {}

  static using(pool: pg.Pool): QueryDatabase {
    return new QueryDatabase(pool);
  }

  async query<T extends pg.QueryResultRow = Record<string, unknown>>(
    sql: string,
    params?: unknown[],
  ): Promise<pg.QueryResult<T>> {
    return this.pool.query<T>(sql, params);
  }

  async queryOne<T extends pg.QueryResultRow = Record<string, unknown>>(
    sql: string,
    params?: unknown[],
  ): Promise<T | null> {
    const result = await this.pool.query<T>(sql, params);
    return result.rows[0] ?? null;
  }

  async count(table: string, where?: string, params?: unknown[]): Promise<number> {
    const sql = where
      ? `SELECT COUNT(*)::int AS count FROM ${table} WHERE ${where}`
      : `SELECT COUNT(*)::int AS count FROM ${table}`;
    const result = await this.pool.query<{ count: number }>(sql, params);
    return result.rows[0]?.count ?? 0;
  }
}
