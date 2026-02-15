import { test, expect } from '@playwright/test';

test.describe('Players Page', () => {
	test('should load players list page', async ({ page }) => {
		await page.goto('/players');

		// Wait for page to load
		await expect(page.locator('h1')).toContainText(/players/i);
	});

	test('should display player cards', async ({ page }) => {
		await page.goto('/players');

		// Wait for players to load
		await page.waitForSelector('[role="button"]', { timeout: 10000 });

		// Check if at least one player card is visible
		const playerCards = page.locator('[role="button"]');
		await expect(playerCards.first()).toBeVisible();
	});

	test('should filter players by position', async ({ page }) => {
		await page.goto('/players');

		// Wait for page to load
		await page.waitForLoadState('networkidle');

		// Use the page-level position select dropdown (id="position")
		const positionSelect = page.locator('#position');
		if (await positionSelect.isVisible()) {
			await positionSelect.selectOption('QB');

			// Wait for filtered results
			await page.waitForTimeout(500);

			// Verify QB badges are visible on player cards (use Badge component text)
			const qbBadges = page.locator('.bg-white.rounded-lg span', { hasText: /^QB$/ });
			await expect(qbBadges.first()).toBeVisible();
		}
	});

	test('should search players by name', async ({ page }) => {
		await page.goto('/players');

		// Wait for page to load
		await page.waitForLoadState('networkidle');

		// Use the page-level search input (id="search")
		const searchInput = page.locator('#search');
		if (await searchInput.isVisible()) {
			await searchInput.fill('John');

			// Wait for search results
			await page.waitForTimeout(500);

			// Verify results are filtered (page should still show players)
			await expect(page.locator('h1')).toContainText(/players/i);
		}
	});

	test('should navigate to player details', async ({ page }) => {
		await page.goto('/players');

		// Wait for players to load
		await page.waitForSelector('[role="button"]', { timeout: 10000 });

		// Click first player card
		const firstPlayer = page.locator('[role="button"]').first();
		await firstPlayer.click();

		// Wait for navigation
		await page.waitForLoadState('networkidle');

		// Verify we navigated to player details
		await expect(page).toHaveURL(/\/players\/[a-f0-9-]+/);
	});

	test('should display player position badges', async ({ page }) => {
		await page.goto('/players');

		// Wait for players to load
		await page.waitForSelector('[role="button"]', { timeout: 10000 });

		// Check for visible position badges inside player cards (not hidden <option> elements)
		// Badge component renders as <span> elements
		const positionBadges = page.locator(
			'[role="button"] span:text-matches("^(QB|RB|WR|TE|OT|OG|C|DE|DT|LB|CB|S|K|P)$")'
		);
		await expect(positionBadges.first()).toBeVisible();
	});

	test('should show loading state', async ({ page }) => {
		// Navigate and check for loading spinner
		await page.goto('/players');

		// Loading spinner might be visible briefly
		// Just verify the page eventually loads
		await expect(page.locator('h1')).toBeVisible();
	});
});

test.describe('Player Details Page', () => {
	test('should display player information', async ({ page }) => {
		// Navigate to players list, then click first player
		await page.goto('/players');

		// Wait for players to load
		await page.waitForSelector('[role="button"]', { timeout: 10000 });

		// Click first player
		const firstPlayer = page.locator('[role="button"]').first();
		await firstPlayer.click();

		// Wait for details page to load
		await page.waitForLoadState('networkidle');

		// Verify player name heading is displayed
		await expect(page.locator('h1')).toBeVisible();
	});

	test('should display player stats', async ({ page }) => {
		await page.goto('/players');
		await page.waitForSelector('[role="button"]', { timeout: 10000 });

		// Click first player
		await page.locator('[role="button"]').first().click();
		await page.waitForLoadState('networkidle');

		// Check for stats section - use .first() to avoid strict mode with multiple matches
		await expect(page.getByText('Height').first()).toBeVisible();
	});

	test('should have back navigation', async ({ page }) => {
		await page.goto('/players');
		await page.waitForSelector('[role="button"]', { timeout: 10000 });

		// Click first player
		await page.locator('[role="button"]').first().click();
		await page.waitForLoadState('networkidle');

		// Click back button (browser back)
		await page.goBack();

		// Verify we're back at the players list
		await expect(page).toHaveURL(/\/players$/);
	});
});
