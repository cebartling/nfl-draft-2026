import type { FullConfig } from '@playwright/test';
import { cleanupTestDrafts } from './src/db/cleanup.js';
import { closePool } from './src/db/client.js';

async function globalTeardown(_config: FullConfig): Promise<void> {
  console.log('Global teardown: cleaning up test data...');

  try {
    await cleanupTestDrafts();
    console.log('  Test drafts cleaned up.');
  } catch (err) {
    console.warn('  Warning: cleanup failed:', err);
  }

  try {
    await closePool();
    console.log('  Database pool closed.');
  } catch (err) {
    console.warn('  Warning: pool close failed:', err);
  }

  console.log('Global teardown: complete.');
}

export default globalTeardown;
