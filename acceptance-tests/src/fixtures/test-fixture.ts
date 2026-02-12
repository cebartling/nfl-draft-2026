import { test as base } from '@playwright/test';
import { Actor } from '../screenplay/actor.js';
import { BrowseTheWeb } from '../screenplay/abilities/browse-the-web.js';
import { CallApi } from '../screenplay/abilities/call-api.js';
import { QueryDatabase } from '../screenplay/abilities/query-database.js';
import { getPool } from '../db/client.js';

type Fixtures = {
  actor: Actor;
};

export const test = base.extend<Fixtures>({
  actor: async ({ page }, use) => {
    const actor = new Actor('Tester').whoCan(
      BrowseTheWeb.using(page),
      CallApi.at(process.env.API_URL || 'http://localhost:8000'),
      QueryDatabase.using(getPool()),
    );
    await use(actor);
  },
});

export { expect } from '@playwright/test';
