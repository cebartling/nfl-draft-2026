import { test, expect } from '../src/fixtures/test-fixture.js';
import { Navigate } from '../src/screenplay/tasks/navigate.js';
import { PageHeading } from '../src/screenplay/questions/web-questions.js';
import { TeamCount } from '../src/screenplay/questions/team-questions.js';
import { CallApi } from '../src/screenplay/abilities/call-api.js';

test.describe('Smoke Tests', () => {
  test('frontend serves the home page', async ({ actor }) => {
    await actor.attemptsTo(Navigate.toHomePage());
    const heading = await actor.asks(PageHeading.text());
    expect(heading).toContain('NFL Draft');
  });

  test('API health endpoint responds', async ({ actor }) => {
    const api = actor.abilityTo(CallApi);
    const response = await api.get<{ status: string }>('/health');
    expect(response.ok).toBe(true);
    expect(response.status).toBe(200);
    expect(response.data).toHaveProperty('status');
    expect(response.data.status).toBe('healthy');
  });

  test('database has seeded teams', async ({ actor }) => {
    const count = await actor.asks(TeamCount.inDatabase());
    expect(count).toBe(32);
  });
});
