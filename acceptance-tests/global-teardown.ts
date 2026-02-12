import type { FullConfig } from '@playwright/test';
import { cleanupTestDrafts } from './src/db/cleanup.js';
import { closePool } from './src/db/client.js';

async function globalTeardown(_config: FullConfig): Promise<void> {
  console.log('Global teardown: cleaning up test data...');

  try {
    await cleanupTestDrafts();
    console.log('  Test drafts cleaned up.');
  } catch (err) {
    // Re-throw so orphaned test data doesn't silently accumulate
    await closePool().catch(() => {});
    throw new Error(`Global teardown: cleanup failed: ${err}`);
  }

  try {
    await closePool();
    console.log('  Database pool closed.');
  } catch (err) {
    // Pool close failures are non-fatal; connections are cleaned up on process exit
    console.warn('  Warning: pool close failed:', err);
  }

  console.log('Global teardown: complete.');
}

export default globalTeardown;
