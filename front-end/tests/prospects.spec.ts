import { test, expect } from '@playwright/test';

test.describe('Prospects Page', () => {
	test('should load prospects page with heading', async ({ page }) => {
		await page.goto('/prospects');

		await expect(page.locator('h1')).toContainText(/prospect rankings/i);
	});

	test('should display ranked prospect count and source count in header', async ({ page }) => {
		await page.goto('/prospects');

		// Wait for rankings to load (non-blocking fetch)
		await page.waitForLoadState('networkidle');

		// Header should show "X ranked prospects | Y sources"
		const headerStats = page.locator('text=/\\d+ ranked prospects/');
		await expect(headerStats).toBeVisible({ timeout: 10000 });

		const sourcesText = page.locator('text=/\\d+ sources/');
		await expect(sourcesText).toBeVisible();
	});

	test('should display rankings table with rows', async ({ page }) => {
		await page.goto('/prospects');

		// Wait for table to appear
		await page.waitForSelector('table', { timeout: 10000 });

		// Table should have header columns
		await expect(page.locator('th', { hasText: '#' })).toBeVisible();
		await expect(page.locator('th', { hasText: 'Player' })).toBeVisible();
		await expect(page.locator('th', { hasText: 'Pos' })).toBeVisible();
		await expect(page.locator('th', { hasText: 'Big Board Rankings' })).toBeVisible();

		// Should have at least one data row
		const rows = page.locator('tbody tr');
		await expect(rows.first()).toBeVisible();
	});

	test('should display ranking badges for prospects', async ({ page }) => {
		await page.goto('/prospects');

		// Wait for table to appear
		await page.waitForSelector('table', { timeout: 10000 });

		// Purple ranking badges should be visible
		const badges = page.locator('.bg-purple-100');
		await expect(badges.first()).toBeVisible({ timeout: 10000 });
	});

	test('should display consensus rank numbers', async ({ page }) => {
		await page.goto('/prospects');

		// Wait for table to appear
		await page.waitForSelector('table', { timeout: 10000 });

		// First row should have rank "1" and an "avg" subtitle
		const firstRow = page.locator('tbody tr').first();
		await expect(firstRow.locator('text=avg')).toBeVisible();
	});

	test('should filter prospects by search query', async ({ page }) => {
		await page.goto('/prospects');

		// Wait for table to load
		await page.waitForSelector('table', { timeout: 10000 });

		// Get the name of the first prospect to search for
		const firstRowName = await page.locator('tbody tr').first().locator('td').nth(1).textContent();
		const searchTerm = firstRowName?.trim().split(/\s+/).pop() || '';

		if (searchTerm) {
			// Type into search
			const searchInput = page.locator('input[placeholder*="search" i]');
			await searchInput.fill(searchTerm);

			// Active filter badge should appear (reactive, no delay needed)
			await expect(page.locator('text=/Search:/i')).toBeVisible();

			// Results should contain the search term
			const resultText = await page.locator('tbody').textContent();
			expect(resultText?.toLowerCase()).toContain(searchTerm.toLowerCase());
		}
	});

	test('should filter prospects by position', async ({ page }) => {
		await page.goto('/prospects');

		// Wait for table to load
		await page.waitForSelector('table', { timeout: 10000 });

		// Select QB position
		const positionSelect = page.locator('select#position');
		await positionSelect.selectOption('QB');

		// Active filter badge should appear (reactive, no delay needed)
		await expect(page.locator('text=/Position: QB/i')).toBeVisible();

		// All visible position badges should be QB
		const rows = page.locator('tbody tr');
		const rowCount = await rows.count();

		if (rowCount > 0) {
			for (let i = 0; i < Math.min(rowCount, 5); i++) {
				const posCell = rows.nth(i).locator('td').nth(2);
				await expect(posCell).toContainText('QB');
			}
		}
	});

	test('should clear filters', async ({ page }) => {
		await page.goto('/prospects');

		// Wait for table to load
		await page.waitForSelector('table', { timeout: 10000 });

		// Apply a filter
		const searchInput = page.locator('input[placeholder*="search" i]');
		await searchInput.fill('test-nonexistent-xyz');

		// Wait for empty state to appear
		await expect(page.locator('text=/no ranked prospects/i')).toBeVisible();

		// Click "Clear all"
		await page.locator('text=Clear all').click();

		// Search should be empty again
		await expect(searchInput).toHaveValue('');

		// Table should have rows again
		const rows = page.locator('tbody tr');
		await expect(rows.first()).toBeVisible();
	});

	test('should navigate to player detail when clicking a row', async ({ page }) => {
		await page.goto('/prospects');

		// Wait for table to load
		await page.waitForSelector('table', { timeout: 10000 });

		// Click first row
		const firstRow = page.locator('tbody tr[role="button"]').first();
		await firstRow.click();

		// Should navigate to a player detail page
		await page.waitForLoadState('networkidle');
		await expect(page).toHaveURL(/\/players\/[a-f0-9-]+/);
	});

	test('should show unranked player count with link to /players', async ({ page }) => {
		await page.goto('/prospects');

		// Wait for rankings to load
		await page.waitForLoadState('networkidle');
		await page.waitForSelector('table', { timeout: 10000 });

		// Footer should mention additional unranked players
		const unrankedLink = page.locator('a[href="/players"]', { hasText: /view all players/i });
		await expect(unrankedLink).toBeVisible({ timeout: 10000 });
	});
});

test.describe('Prospects Navigation', () => {
	test('should have Prospects link in desktop navigation', async ({ page }) => {
		await page.goto('/');

		const navLink = page.locator('nav a[href="/prospects"]');
		await expect(navLink).toBeVisible();
		await expect(navLink).toContainText('Prospects');
	});

	test('should navigate to prospects page via nav link', async ({ page }) => {
		await page.goto('/');

		await page.locator('nav a[href="/prospects"]').click();
		await page.waitForLoadState('networkidle');

		await expect(page).toHaveURL('/prospects');
		await expect(page.locator('h1')).toContainText(/prospect rankings/i);
	});
});
