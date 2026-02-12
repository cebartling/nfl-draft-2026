import { getPool } from './client.js';

const VALID_IDENTIFIER = /^[a-z_][a-z0-9_]*$/;

/**
 * Allowed tables for cleanup. Acts as an allowlist to prevent accidental
 * deletion of reference data (teams, players, rankings, etc.).
 *
 * NOTE: This intentionally diverges from the backend Rust cleanup
 * (back-end/crates/api/tests/common/mod.rs) which deletes ALL tables
 * including teams/players. Here we only delete draft-related data
 * because acceptance tests rely on seeded reference data.
 */
const DRAFT_CLEANUP_TABLES = [
  'pick_trade_details',
  'pick_trades',
  'draft_strategies',
  'draft_events',
  'draft_sessions',
  'draft_picks',
  'drafts',
] as const;

/**
 * Deletes draft-related data in FK dependency order.
 * Preserves seeded reference data (teams, players, rankings, etc.).
 */
export async function cleanupTestDrafts(): Promise<void> {
  const pool = getPool();

  for (const table of DRAFT_CLEANUP_TABLES) {
    if (!VALID_IDENTIFIER.test(table)) {
      throw new Error(`Unsafe table name in cleanup allowlist: "${table}"`);
    }
    await pool.query(`DELETE FROM ${table}`);
  }
}
