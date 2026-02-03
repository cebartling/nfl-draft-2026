# Frontend Test Suite

This directory contains the comprehensive test suite for the NFL Draft Simulator 2026 frontend.

## Test Structure

### Unit Tests (API Layer)

Located in `src/lib/api/*.test.ts`

- **client.test.ts** - Tests for ApiClient class
  - GET/POST/PUT/DELETE requests
  - Error handling (404, 500, network errors)
  - Zod schema validation
  - ApiClientError class

- **teams.test.ts** - Tests for teams API module
  - list(), get(), create(), update()
  - getNeeds(), createNeed()
  - Mock apiClient with vi.spyOn()

- **players.test.ts** - Tests for players API module
  - list(), get(), create(), getByPosition()
  - getScoutingReports(), createScoutingReport()
  - getCombineResults() with 404 handling

- **websocket.test.ts** - Tests for WebSocket client
  - Connection lifecycle
  - Message sending and receiving
  - Automatic reconnection with exponential backoff
  - Event handlers and state management
  - Ping/pong keepalive

### Unit Tests (State Management)

Located in `src/lib/stores/*.test.ts`

- **toast.test.ts** - Tests for toast notification state
  - Show/hide toasts
  - Auto-dismiss after duration
  - Toast types (success, error, info, warning)

### Component Tests

Located in `src/lib/components/**/*.test.ts`

- **ui/Button.test.ts** - Tests for Button component
  - Variants (primary, secondary, danger)
  - Sizes (sm, md, lg)
  - Disabled and loading states
  - Click handling

### E2E Tests (Playwright)

Located in `tests/*.spec.ts`

- **home.spec.ts** - Home page tests
  - Page loads successfully
  - Navigation links work
  - Responsive design

- **players.spec.ts** - Players page tests
  - Player list loads
  - Position filtering
  - Search functionality
  - Navigation to player details

- **teams.spec.ts** - Teams page tests
  - Team list loads
  - Conference/division grouping
  - Navigation to team details
  - Team needs display

## Running Tests

### Unit and Component Tests (Vitest)

```bash
# Run all tests
npm run test

# Run tests in watch mode
npm run test:watch

# Run tests with UI
npm run test:ui

# Run tests with coverage
npm run test -- --coverage
```

### E2E Tests (Playwright)

**Prerequisites:**

- Backend server must be running on `http://localhost:8000`
- Database must be seeded with test data

```bash
# Run all E2E tests
npm run test:e2e

# Run specific test file
npm run test:e2e tests/home.spec.ts

# Run tests in headed mode (see browser)
npm run test:e2e -- --headed

# Run tests in debug mode
npm run test:e2e -- --debug

# Run tests on specific browser
npm run test:e2e -- --project=chromium
npm run test:e2e -- --project=firefox
npm run test:e2e -- --project=webkit
```

## Test Fixtures

Test fixtures are located in `tests/fixtures.ts` and provide:

- **mockTeams** - 8 sample NFL teams (AFC East + AFC North)
- **mockPlayers** - 5 sample players with different positions
- **mockDraft** - Sample 2026 draft
- **mockDraftPicks** - Sample draft picks
- **mockTradeProposal** - Sample trade proposal
- **generateDraftPicks()** - Helper to generate full draft board
- **createMockSession()** - Helper to create draft session

## Mocking Strategy

### API Tests

- Use `vi.spyOn()` to mock apiClient methods
- Mock fetch with `vi.fn()`
- Create mock responses matching Zod schemas

### WebSocket Tests

- Mock WebSocket class with custom implementation
- Simulate connection events (open, close, error)
- Simulate message events with test data

### Component Tests

- Use `@testing-library/svelte` for rendering
- Use `@testing-library/user-event` for interactions
- Mock stores and API modules as needed

### E2E Tests

- Use real backend API (requires backend running)
- Seed database with known test data
- Clean up test data after tests

## CI/CD Integration

Tests are configured to run in CI with:

- Retries: 2 attempts for flaky tests
- Single worker for database stability
- Screenshots on failure
- Video recording on failure
- HTML report generation

## Writing New Tests

### Unit Test Example

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';

describe('MyModule', () => {
	beforeEach(() => {
		// Setup
	});

	it('should do something', () => {
		// Arrange
		const input = 'test';

		// Act
		const result = myFunction(input);

		// Assert
		expect(result).toBe('expected');
	});
});
```

### Component Test Example

```typescript
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';

it('should render component', async () => {
	const user = userEvent.setup();
	render(MyComponent, { props: { name: 'Test' } });

	const button = screen.getByRole('button');
	await user.click(button);

	expect(screen.getByText('Clicked')).toBeInTheDocument();
});
```

### E2E Test Example

```typescript
import { test, expect } from '@playwright/test';

test('should navigate to page', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('link', { name: 'Teams' }).click();
	await expect(page).toHaveURL('/teams');
});
```

## Troubleshooting

### Tests Failing Locally

1. **Backend not running** - Start backend server on port 8000
2. **Database not seeded** - Run migrations and seed scripts
3. **Port conflicts** - Ensure ports 5173 and 8000 are available
4. **Stale cache** - Clear node_modules and reinstall

### E2E Tests Timing Out

1. Increase timeout in `playwright.config.ts`
2. Check backend logs for errors
3. Verify WebSocket connection working
4. Run in headed mode to debug: `npm run test:e2e -- --headed`

### WebSocket Tests Failing

1. Ensure mock WebSocket is properly set up
2. Check for race conditions in async code
3. Use `vi.useFakeTimers()` for time-dependent tests

## Coverage Goals

- **API Layer**: 100% coverage (critical path)
- **State Management**: 95%+ coverage
- **Components**: 80%+ coverage (focus on interactive components)
- **E2E**: Critical user flows (happy path + error cases)

## Future Improvements

- [ ] Add visual regression testing with Percy or Chromatic
- [ ] Add performance testing with Lighthouse
- [ ] Add accessibility testing with axe-core
- [ ] Add API contract testing
- [ ] Add load testing for WebSocket connections
- [ ] Add mutation testing for API layer
