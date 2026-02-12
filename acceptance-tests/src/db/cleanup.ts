import { getPool } from './client.js';

/**
 * Name prefix used by all acceptance test drafts. Cleanup is scoped to
 * drafts matching this prefix so manually-created dev drafts are preserved.
 */
export const TEST_DRAFT_PREFIX = 'E2E ';

/**
 * Deletes only acceptance-test-created draft data in FK dependency order.
 * Scoped to drafts whose name starts with TEST_DRAFT_PREFIX ("E2E ").
 * Preserves all other data including manually-created dev drafts.
 *
 * FK chain: pick_trade_details → pick_trades → draft_sessions → drafts
 *           draft_events → draft_sessions → drafts
 *           draft_strategies → drafts
 *           draft_picks → drafts
 *
 * NOTE: This intentionally diverges from the backend Rust cleanup
 * (back-end/crates/api/tests/common/mod.rs) which deletes ALL tables
 * including teams/players. Here we only delete test-created draft data
 * because acceptance tests rely on seeded reference data.
 */
export async function cleanupTestDrafts(): Promise<void> {
  const pool = getPool();

  // Find draft IDs created by acceptance tests
  const { rows: drafts } = await pool.query<{ id: string }>(
    `SELECT id FROM drafts WHERE name LIKE $1`,
    [`${TEST_DRAFT_PREFIX}%`],
  );

  if (drafts.length === 0) return;

  const draftIds = drafts.map((d) => d.id);

  // Delete in FK dependency order, scoped to test draft IDs
  // pick_trade_details → pick_trades (via trade_id) → draft_sessions (via session_id)
  await pool.query(
    `DELETE FROM pick_trade_details WHERE trade_id IN (
       SELECT id FROM pick_trades WHERE session_id IN (
         SELECT id FROM draft_sessions WHERE draft_id = ANY($1)
       )
     )`,
    [draftIds],
  );
  // pick_trades → draft_sessions (via session_id)
  await pool.query(
    `DELETE FROM pick_trades WHERE session_id IN (
       SELECT id FROM draft_sessions WHERE draft_id = ANY($1)
     )`,
    [draftIds],
  );
  await pool.query(`DELETE FROM draft_strategies WHERE draft_id = ANY($1)`, [draftIds]);
  await pool.query(
    `DELETE FROM draft_events WHERE session_id IN (
       SELECT id FROM draft_sessions WHERE draft_id = ANY($1)
     )`,
    [draftIds],
  );
  await pool.query(`DELETE FROM draft_sessions WHERE draft_id = ANY($1)`, [draftIds]);
  await pool.query(`DELETE FROM draft_picks WHERE draft_id = ANY($1)`, [draftIds]);
  await pool.query(`DELETE FROM drafts WHERE id = ANY($1)`, [draftIds]);
}
