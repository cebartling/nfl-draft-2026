import type pg from 'pg';
import type { Ability } from '../actor.js';

const VALID_IDENTIFIER = /^[a-z_][a-z0-9_]*$/;

function assertSafeIdentifier(value: string, label: string): void {
  if (!VALID_IDENTIFIER.test(value)) {
    throw new Error(
      `Unsafe SQL identifier for ${label}: "${value}". ` +
        `Only lowercase letters, digits, and underscores are allowed.`,
    );
  }
}

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

  /**
   * Count rows in a table with an optional WHERE clause.
   *
   * @param table - Table name (must be a safe SQL identifier: lowercase, digits, underscores)
   * @param where - Optional WHERE clause using $1, $2, etc. for parameters
   * @param params - Optional bind parameters for the WHERE clause
   */
  async count(table: string, where?: string, params?: unknown[]): Promise<number> {
    assertSafeIdentifier(table, 'table');
    const sql = where
      ? `SELECT COUNT(*)::int AS count FROM ${table} WHERE ${where}`
      : `SELECT COUNT(*)::int AS count FROM ${table}`;
    const result = await this.pool.query<{ count: number }>(sql, params);
    return result.rows[0]?.count ?? 0;
  }
}
