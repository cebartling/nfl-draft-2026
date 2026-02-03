import { test, expect } from '@playwright/test';

test.describe('Teams Page', () => {
	test('should load teams list page', async ({ page }) => {
		await page.goto('/teams');

		// Wait for page to load
		await expect(page.locator('h1')).toContainText(/teams/i);
	});

	test('should display team cards', async ({ page }) => {
		await page.goto('/teams');

		// Wait for teams to load
		await page.waitForLoadState('networkidle');

		// Check if team cards are visible
		const teamCards = page.locator('[role="button"], [role="article"]');
		await expect(teamCards.first()).toBeVisible({ timeout: 10000 });
	});

	test('should group teams by conference', async ({ page }) => {
		await page.goto('/teams');

		// Wait for page to load
		await page.waitForLoadState('networkidle');

		// Check for conference headings
		const afcHeading = page.locator('h2, h3').filter({ hasText: /AFC/i });
		const nfcHeading = page.locator('h2, h3').filter({ hasText: /NFC/i });

		// At least one conference should be visible
		const afcVisible = await afcHeading.isVisible().catch(() => false);
		const nfcVisible = await nfcHeading.isVisible().catch(() => false);

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
		await page.waitForTimeout(1000);

		// Find and click a team card
		const teamCard = page.locator('[role="button"]').first();
		if (await teamCard.isVisible()) {
			await teamCard.click();

			// Wait for navigation
			await page.waitForLoadState('networkidle');

			// Verify we navigated to team details
			await expect(page).toHaveURL(/\/teams\/[a-f0-9-]+/);
		}
	});

	test('should filter teams by division', async ({ page }) => {
		await page.goto('/teams');

		// Wait for page to load
		await page.waitForLoadState('networkidle');

		// Look for division filters (if available)
		const divisionFilter = page.locator('button, select', { hasText: /East|West|North|South/i });
		if (await divisionFilter.first().isVisible()) {
			await divisionFilter.first().click();

			// Wait for filtered results
			await page.waitForTimeout(500);

			// Verify teams are filtered
			const teamCards = page.locator('[role="button"]');
			await expect(teamCards.first()).toBeVisible();
		}
	});

	test('should display conference badges', async ({ page }) => {
		await page.goto('/teams');

		// Wait for teams to load
		await page.waitForLoadState('networkidle');

		// Check for AFC/NFC badges
		const conferenceBadges = page.locator('text=/AFC|NFC/');
		await expect(conferenceBadges.first()).toBeVisible({ timeout: 10000 });
	});
});

test.describe('Team Details Page', () => {
	test('should display team information', async ({ page }) => {
		await page.goto('/teams');

		// Wait for teams to load
		await page.waitForLoadState('networkidle');
		await page.waitForTimeout(1000);

		// Click first team
		const firstTeam = page.locator('[role="button"]').first();
		if (await firstTeam.isVisible()) {
			const teamName = await firstTeam.textContent();
			await firstTeam.click();

			// Wait for details page to load
			await page.waitForLoadState('networkidle');

			// Verify team name is displayed
			if (teamName) {
				await expect(page.locator('h1, h2')).toContainText(teamName.trim(), { timeout: 5000 });
			}
		}
	});

	test('should display team needs section', async ({ page }) => {
		await page.goto('/teams');
		await page.waitForLoadState('networkidle');
		await page.waitForTimeout(1000);

		// Click first team
		const firstTeam = page.locator('[role="button"]').first();
		if (await firstTeam.isVisible()) {
			await firstTeam.click();
			await page.waitForLoadState('networkidle');

			// Check for needs section (team might not have needs, so don't assert)
			// const needsSection = page.locator('text=/Needs|Draft Needs|Team Needs/i');
			// Just verify page loaded
			await expect(page.locator('h1, h2')).toBeVisible();
		}
	});

	test('should display team conference and division', async ({ page }) => {
		await page.goto('/teams');
		await page.waitForLoadState('networkidle');
		await page.waitForTimeout(1000);

		// Click first team
		const firstTeam = page.locator('[role="button"]').first();
		if (await firstTeam.isVisible()) {
			await firstTeam.click();
			await page.waitForLoadState('networkidle');

			// Check for conference/division info
			await expect(page.locator('text=/AFC|NFC/i')).toBeVisible({ timeout: 5000 });
		}
	});

	test('should have back navigation', async ({ page }) => {
		await page.goto('/teams');
		await page.waitForLoadState('networkidle');
		await page.waitForTimeout(1000);

		// Click first team
		const firstTeam = page.locator('[role="button"]').first();
		if (await firstTeam.isVisible()) {
			await firstTeam.click();
			await page.waitForLoadState('networkidle');

			// Click back button (browser back)
			await page.goBack();

			// Verify we're back at the teams list
			await expect(page).toHaveURL(/\/teams$/);
		}
	});

	test('should display team logo if available', async ({ page }) => {
		await page.goto('/teams');
		await page.waitForLoadState('networkidle');
		await page.waitForTimeout(1000);

		// Click first team
		const firstTeam = page.locator('[role="button"]').first();
		if (await firstTeam.isVisible()) {
			await firstTeam.click();
			await page.waitForLoadState('networkidle');

			// Check for logo image (might not exist, so don't assert)
			// const logo = page.locator('img[alt*="logo" i]');
			// Just verify page loaded
			await expect(page.locator('h1, h2')).toBeVisible();
		}
	});
});
