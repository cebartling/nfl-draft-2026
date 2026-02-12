import type { FullConfig } from '@playwright/test';
import { getPool, closePool } from './src/db/client.js';

async function globalSetup(_config: FullConfig): Promise<void> {
  const frontendUrl = process.env.FRONTEND_URL || 'http://localhost:3000';
  const apiUrl = process.env.API_URL || 'http://localhost:8000';
  let poolCreated = false;

  console.log('Global setup: verifying services are healthy...');

  try {
    // Check frontend (nginx)
    try {
      const res = await fetch(`${frontendUrl}/health`);
      if (!res.ok) throw new Error(`Frontend returned ${res.status}`);
      console.log('  Frontend (nginx): healthy');
    } catch (err) {
      throw new Error(
        `Frontend is not reachable at ${frontendUrl}/health. ` +
          `Ensure containers are running: docker compose up -d --build postgres api frontend\n` +
          `Original error: ${err}`,
      );
    }

    // Check API
    try {
      const res = await fetch(`${apiUrl}/health`);
      if (!res.ok) throw new Error(`API returned ${res.status}`);
      console.log('  API: healthy');
    } catch (err) {
      throw new Error(
        `API is not reachable at ${apiUrl}/health. ` +
          `Ensure containers are running: docker compose up -d --build postgres api frontend\n` +
          `Original error: ${err}`,
      );
    }

    // Check database
    try {
      const pool = getPool();
      poolCreated = true;
      const result = await pool.query('SELECT 1 AS ok');
      if (result.rows[0]?.ok !== 1) throw new Error('Unexpected query result');
      console.log('  Database: connected');
    } catch (err) {
      throw new Error(
        `Database is not reachable. Ensure PostgreSQL is running and seeded.\n` +
          `Original error: ${err}`,
      );
    }

    console.log('Global setup: all services healthy.');
  } catch (err) {
    // Close the pool if we created it before failing, to avoid leaked connections
    if (poolCreated) {
      await closePool();
    }
    throw err;
  }
}

export default globalSetup;
