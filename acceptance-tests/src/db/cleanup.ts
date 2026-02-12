import { getPool } from './client.js';

/**
 * Deletes draft-related data in FK dependency order.
 * Preserves seeded reference data (teams, players, rankings, etc.).
 *
 * Mirrors the cleanup order from back-end/crates/api/tests/common/mod.rs.
 */
export async function cleanupTestDrafts(): Promise<void> {
  const pool = getPool();

  const tables = [
    'pick_trade_details',
    'pick_trades',
    'draft_strategies',
    'draft_events',
    'draft_sessions',
    'draft_picks',
    'drafts',
  ];

  for (const table of tables) {
    await pool.query(`DELETE FROM ${table}`);
  }
}
