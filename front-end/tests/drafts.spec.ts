import { test, expect } from '@playwright/test';

test.describe('Drafts Page', () => {
	test('should load drafts page successfully', async ({ page }) => {
		await page.goto('/drafts');

		await expect(page.getByRole('heading', { name: /Drafts/i })).toBeVisible();
	});

	test('should display Create New Draft button', async ({ page }) => {
		await page.goto('/drafts');

		await expect(page.getByRole('button', { name: /Create New Draft/i })).toBeVisible();
	});

	test('should have create draft page accessible', async ({ page }) => {
		await page.goto('/drafts/new');

		await expect(page.getByRole('heading', { name: /Create New Draft/i })).toBeVisible();
	});

	test('should display status filters', async ({ page }) => {
		await page.goto('/drafts');

		// Wait for loading to complete
		await page.waitForLoadState('networkidle');

		// Use exact button elements (not role="button" cards) for filter buttons
		await expect(page.locator('button', { hasText: /^All/ })).toBeVisible();
		await expect(page.locator('button', { hasText: /^Pending/ })).toBeVisible();
		await expect(page.locator('button', { hasText: /^Active/ })).toBeVisible();
		await expect(page.locator('button', { hasText: /^Completed/ })).toBeVisible();
	});
});

test.describe('Create Draft Page', () => {
	test('should load create draft page', async ({ page }) => {
		await page.goto('/drafts/new');

		await expect(page.getByRole('heading', { name: /Create New Draft/i })).toBeVisible();
	});

	test('should display form with draft name and rounds', async ({ page }) => {
		await page.goto('/drafts/new');

		// Check Draft Name field exists
		await expect(page.getByLabel(/Draft Name/i)).toBeVisible();
		await expect(page.getByLabel(/Draft Name/i)).toHaveValue(/.+/); // Has auto-generated name

		// Check Number of Rounds slider exists
		await expect(page.getByLabel(/Number of Rounds/i)).toBeVisible();
	});

	test('should display draft summary', async ({ page }) => {
		await page.goto('/drafts/new');

		// Summary shows Year (use exact match to avoid matching nav logo "Draft 2026")
		await expect(page.getByText('2026', { exact: true })).toBeVisible();
		await expect(page.getByText('Rounds', { exact: true })).toBeVisible();
		await expect(page.getByText('Realistic')).toBeVisible();
	});

	test('should display Cancel and Back to Drafts buttons', async ({ page }) => {
		await page.goto('/drafts/new');

		await expect(page.getByRole('button', { name: /Cancel/i })).toBeVisible();
		await expect(page.getByRole('button', { name: /Back to Drafts/i })).toBeVisible();
	});

	test('should display Create Draft submit button', async ({ page }) => {
		await page.goto('/drafts/new');

		await expect(page.getByRole('button', { name: /Create Draft/i })).toBeVisible();
	});
});

test.describe('Draft Detail Page', () => {
	test('should display draft detail when navigating from list', async ({ page }) => {
		await page.goto('/drafts');

		// Wait for drafts to load
		await page.waitForLoadState('networkidle');

		// Click on the first draft card (Card component with role="button")
		const draftCard = page.locator('[role="button"]').filter({ hasText: /Draft|NFL/i }).first();
		if (await draftCard.isVisible({ timeout: 5000 }).catch(() => false)) {
			await draftCard.click();

			// Should navigate to a draft detail page
			await expect(page).toHaveURL(/\/drafts\/[a-f0-9-]+/, { timeout: 10000 });

			// Should show draft header
			await expect(page.getByRole('heading', { name: /NFL Draft/i })).toBeVisible();
		}
	});

	test('should display draft details', async ({ page }) => {
		await page.goto('/drafts');
		await page.waitForLoadState('networkidle');

		const draftCard = page.locator('[role="button"]').filter({ hasText: /Draft|NFL/i }).first();
		if (await draftCard.isVisible({ timeout: 5000 }).catch(() => false)) {
			await draftCard.click();
			await expect(page).toHaveURL(/\/drafts\/[a-f0-9-]+/, { timeout: 10000 });

			// Should show draft details
			await expect(page.getByText('Rounds')).toBeVisible();
		}
	});

	test('should display draft action buttons', async ({ page }) => {
		await page.goto('/drafts');
		await page.waitForLoadState('networkidle');

		const draftCard = page.locator('[role="button"]').filter({ hasText: /Draft|NFL/i }).first();
		if (await draftCard.isVisible({ timeout: 5000 }).catch(() => false)) {
			await draftCard.click();
			await expect(page).toHaveURL(/\/drafts\/[a-f0-9-]+/, { timeout: 10000 });

			// Should show some action button (Start Draft, Initialize, etc.)
			const hasAction =
				(await page.getByRole('button', { name: /Start Draft/i }).isVisible().catch(() => false)) ||
				(await page.getByRole('button', { name: /Initialize/i }).isVisible().catch(() => false)) ||
				(await page.getByRole('button', { name: /Back/i }).isVisible().catch(() => false));

			expect(hasAction).toBe(true);
		}
	});
});
