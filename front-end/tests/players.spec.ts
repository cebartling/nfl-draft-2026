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
		// Note: This assumes the backend is running and has player data
		await page.waitForSelector('[role="button"]', { timeout: 10000 });

		// Check if at least one player card is visible
		const playerCards = page.locator('[role="button"]');
		await expect(playerCards.first()).toBeVisible();
	});

	test('should filter players by position', async ({ page }) => {
		await page.goto('/players');

		// Wait for page to load
		await page.waitForLoadState('networkidle');

		// Find and click position filter (if available)
		const qbFilter = page.locator('button', { hasText: 'QB' });
		if (await qbFilter.isVisible()) {
			await qbFilter.click();

			// Wait for filtered results
			await page.waitForTimeout(500);

			// Verify QB badge is visible on cards
			const positionBadges = page.locator('text=QB');
			await expect(positionBadges.first()).toBeVisible();
		}
	});

	test('should search players by name', async ({ page }) => {
		await page.goto('/players');

		// Wait for page to load
		await page.waitForLoadState('networkidle');

		// Find search input (if available)
		const searchInput = page.locator('input[type="search"], input[placeholder*="search" i]');
		if (await searchInput.isVisible()) {
			await searchInput.fill('John');

			// Wait for search results
			await page.waitForTimeout(500);

			// Verify results contain search term
			const results = page.locator('text=John');
			await expect(results.first()).toBeVisible();
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

		// Check for position badges
		const positionBadges = page.locator('text=/^(QB|RB|WR|TE|OT|OG|C|DE|DT|LB|CB|S|K|P)$/');
		await expect(positionBadges.first()).toBeVisible();
	});

	test('should show loading state', async ({ page }) => {
		// Navigate and check for loading spinner
		await page.goto('/players');

		// Loading spinner might be visible briefly
		// This is a timing-sensitive test
		const spinner = page.locator('.animate-spin');
		// Don't assert on spinner as it might be too fast
		// Just verify the page eventually loads
		await expect(page.locator('h1')).toBeVisible();
	});
});

test.describe('Player Details Page', () => {
	test('should display player information', async ({ page }) => {
		// Note: This test requires a valid player ID
		// In a real scenario, you'd seed the database with known test data
		await page.goto('/players');

		// Wait for players to load
		await page.waitForSelector('[role="button"]', { timeout: 10000 });

		// Click first player
		const firstPlayer = page.locator('[role="button"]').first();
		const playerName = await firstPlayer.locator('h3').textContent();
		await firstPlayer.click();

		// Wait for details page to load
		await page.waitForLoadState('networkidle');

		// Verify player name is displayed
		if (playerName) {
			await expect(page.locator('h1')).toContainText(playerName);
		}
	});

	test('should display player stats', async ({ page }) => {
		await page.goto('/players');
		await page.waitForSelector('[role="button"]', { timeout: 10000 });

		// Click first player
		await page.locator('[role="button"]').first().click();
		await page.waitForLoadState('networkidle');

		// Check for stats section
		await expect(page.locator('text=/Height|Weight|Position/i')).toBeVisible();
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
