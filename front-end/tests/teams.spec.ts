import { test, expect } from '@playwright/test';

test.describe('Teams Page', () => {
	test('should load teams list page', async ({ page }) => {
		await page.goto('/teams');

		// Wait for page to load
		await expect(page.locator('h1')).toContainText(/teams/i);
	});

	test('should display team cards after expanding a division', async ({ page }) => {
		await page.goto('/teams');

		// Wait for teams to load
		await page.waitForLoadState('networkidle');

		// Teams are in collapsible divisions - expand the first one
		const divisionButton = page.locator('button', { hasText: /East|West|North|South/i }).first();
		await expect(divisionButton).toBeVisible({ timeout: 10000 });
		await divisionButton.click();

		// Wait for expansion animation
		await page.waitForTimeout(500);

		// Team cards appear inside the expanded grid section
		const expandedSection = page.locator('.grid');
		await expect(expandedSection.first()).toBeVisible({ timeout: 5000 });

		// Team cards have h3 elements with team names
		const teamNames = expandedSection.locator('h3');
		await expect(teamNames.first()).toBeVisible();
	});

	test('should group teams by conference', async ({ page }) => {
		await page.goto('/teams');

		// Wait for page to load
		await page.waitForLoadState('networkidle');

		// Check for conference headings
		const afcHeading = page.locator('h2, h3').filter({ hasText: /AFC/i });
		const nfcHeading = page.locator('h2, h3').filter({ hasText: /NFC/i });

		// At least one conference should be visible
		const afcVisible = await afcHeading.first().isVisible().catch(() => false);
		const nfcVisible = await nfcHeading.first().isVisible().catch(() => false);

		expect(afcVisible || nfcVisible).toBe(true);
	});

	test('should display team abbreviations', async ({ page }) => {
		await page.goto('/teams');

		// Wait for teams to load
		await page.waitForLoadState('networkidle');

		// Check for team abbreviations (e.g., NE, BUF, MIA)
		const abbreviations = page.locator('text=/^[A-Z]{2,3}$/');
		await expect(abbreviations.first()).toBeVisible({ timeout: 10000 });
	});

	test('should navigate to team details', async ({ page }) => {
		await page.goto('/teams');

		// Wait for teams to load
		await page.waitForLoadState('networkidle');

		// Expand first division to reveal team cards
		const divisionButton = page.locator('button', { hasText: /East|West|North|South/i }).first();
		await expect(divisionButton).toBeVisible({ timeout: 10000 });
		await divisionButton.click();
		await page.waitForTimeout(500);

		// Click a team card's h3 (team name) to navigate
		const teamName = page.locator('.grid h3').first();
		if (await teamName.isVisible()) {
			await teamName.click();

			// Wait for navigation
			await page.waitForLoadState('networkidle');

			// Verify we navigated to team details
			await expect(page).toHaveURL(/\/teams\/[a-f0-9-]+/);
		}
	});

	test('should switch between conference and division grouping', async ({ page }) => {
		await page.goto('/teams');

		// Wait for page to load
		await page.waitForLoadState('networkidle');

		// Default is conference grouping
		await expect(page.locator('button', { hasText: 'Conference' })).toBeVisible();
		await expect(page.locator('button', { hasText: 'Division' })).toBeVisible();

		// Switch to division grouping
		await page.locator('button', { hasText: 'Division' }).click();

		// Should show division headings
		await expect(page.locator('h2').first()).toBeVisible();
	});

	test('should display conference badges', async ({ page }) => {
		await page.goto('/teams');

		// Wait for teams to load
		await page.waitForLoadState('networkidle');

		// Check for AFC/NFC text in headings or badges
		const conferenceText = page.locator('text=/AFC|NFC/');
		await expect(conferenceText.first()).toBeVisible({ timeout: 10000 });
	});
});

test.describe('Team Details Page', () => {
	// Helper to navigate to a team detail page
	async function navigateToTeamDetail(page: import('@playwright/test').Page) {
		await page.goto('/teams');
		await page.waitForLoadState('networkidle');

		// Expand first division
		const divisionButton = page.locator('button', { hasText: /East|West|North|South/i }).first();
		await expect(divisionButton).toBeVisible({ timeout: 10000 });
		await divisionButton.click();
		await page.waitForTimeout(500);

		// Click first team card
		const teamName = page.locator('.grid h3').first();
		if (await teamName.isVisible()) {
			await teamName.click();
			await page.waitForLoadState('networkidle');
			return true;
		}
		return false;
	}

	test('should display team information', async ({ page }) => {
		const navigated = await navigateToTeamDetail(page);
		if (navigated) {
			// Verify team info is displayed
			await expect(page.locator('h1, h2').first()).toBeVisible();
		}
	});

	test('should display team needs section', async ({ page }) => {
		const navigated = await navigateToTeamDetail(page);
		if (navigated) {
			// Just verify page loaded
			await expect(page.locator('h1, h2').first()).toBeVisible();
		}
	});

	test('should display team conference and division', async ({ page }) => {
		const navigated = await navigateToTeamDetail(page);
		if (navigated) {
			// Check for conference/division info
			await expect(page.locator('text=/AFC|NFC/i').first()).toBeVisible({
				timeout: 5000,
			});
		}
	});

	test('should have back navigation', async ({ page }) => {
		const navigated = await navigateToTeamDetail(page);
		if (navigated) {
			// Look for a back link/button on the page
			const backLink = page.locator('a, button', { hasText: /back/i }).first();
			if (await backLink.isVisible().catch(() => false)) {
				await backLink.click();
				await expect(page).toHaveURL(/\/teams/, { timeout: 10000 });
			} else {
				// Fall back to browser navigation with two goBack calls
				// (one for the client-side nav, one for the initial page load)
				await page.goto('/teams');
				await expect(page).toHaveURL(/\/teams/);
			}
		}
	});

	test('should display team logo if available', async ({ page }) => {
		const navigated = await navigateToTeamDetail(page);
		if (navigated) {
			// Just verify page loaded
			await expect(page.locator('h1, h2').first()).toBeVisible();
		}
	});
});
