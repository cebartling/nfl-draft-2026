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

	test('should navigate to create draft page', async ({ page }) => {
		await page.goto('/drafts');

		await page.getByRole('button', { name: /Create New Draft/i }).click();

		await expect(page).toHaveURL('/drafts/new');
	});

	test('should display status filters', async ({ page }) => {
		await page.goto('/drafts');

		await expect(page.getByRole('button', { name: /All/i })).toBeVisible();
		await expect(page.getByRole('button', { name: /Pending/i })).toBeVisible();
		await expect(page.getByRole('button', { name: /Active/i })).toBeVisible();
		await expect(page.getByRole('button', { name: /Completed/i })).toBeVisible();
	});
});

test.describe('Create Draft Page', () => {
	test('should load create draft page', async ({ page }) => {
		await page.goto('/drafts/new');

		await expect(page.getByRole('heading', { name: /Create New Draft/i })).toBeVisible();
	});

	test('should display form with default values', async ({ page }) => {
		await page.goto('/drafts/new');

		// Check default values
		await expect(page.getByLabel(/Draft Year/i)).toHaveValue('2026');
		await expect(page.getByLabel(/Number of Rounds/i)).toHaveValue('7');
		await expect(page.getByLabel(/Picks per Round/i)).toHaveValue('32');
	});

	test('should display draft summary with total picks', async ({ page }) => {
		await page.goto('/drafts/new');

		// Default: 7 rounds * 32 picks = 224 total
		await expect(page.getByText('224')).toBeVisible();
		await expect(page.getByText('Total Picks')).toBeVisible();
	});

	test('should update total picks when inputs change', async ({ page }) => {
		await page.goto('/drafts/new');

		// Change rounds to 3
		await page.getByLabel(/Number of Rounds/i).fill('3');

		// 3 rounds * 32 picks = 96 total
		await expect(page.getByText('96')).toBeVisible();
	});

	test('should show validation error for invalid year', async ({ page }) => {
		await page.goto('/drafts/new');

		// Enter invalid year
		await page.getByLabel(/Draft Year/i).fill('2050');

		// Check for validation error
		await expect(page.getByText(/Year must be between/i)).toBeVisible();
	});

	test('should show validation error for invalid rounds', async ({ page }) => {
		await page.goto('/drafts/new');

		// Enter invalid rounds
		await page.getByLabel(/Number of Rounds/i).fill('10');

		// Check for validation error
		await expect(page.getByText(/Rounds must be between/i)).toBeVisible();
	});

	test('should disable submit button with validation errors', async ({ page }) => {
		await page.goto('/drafts/new');

		// Enter invalid year
		await page.getByLabel(/Draft Year/i).fill('2050');

		// Submit button should be disabled
		await expect(page.getByRole('button', { name: /Create Draft/i })).toBeDisabled();
	});

	test('should navigate back to drafts page on cancel', async ({ page }) => {
		await page.goto('/drafts/new');

		await page.getByRole('button', { name: /Cancel/i }).click();

		await expect(page).toHaveURL('/drafts');
	});

	test('should navigate back to drafts page via back button', async ({ page }) => {
		await page.goto('/drafts/new');

		await page.getByRole('button', { name: /Back to Drafts/i }).click();

		await expect(page).toHaveURL('/drafts');
	});

	test('should show error for duplicate year draft', async ({ page }) => {
		// First, ensure a 2026 draft exists by checking the drafts page
		await page.goto('/drafts');

		// Look for existing 2026 draft
		const has2026Draft = await page.getByText('2026 Draft').isVisible().catch(() => false);

		if (has2026Draft) {
			// Try to create another 2026 draft
			await page.goto('/drafts/new');
			await page.getByLabel(/Draft Year/i).fill('2026');
			await page.getByRole('button', { name: /Create Draft/i }).click();

			// Should show error message
			await expect(page.getByText(/already exists/i)).toBeVisible({ timeout: 10000 });
		}
	});

	test('should create draft and redirect to detail page', async ({ page }) => {
		await page.goto('/drafts/new');

		// Use a unique year to avoid conflicts
		const uniqueYear = 2029;
		await page.getByLabel(/Draft Year/i).fill(String(uniqueYear));
		await page.getByLabel(/Number of Rounds/i).fill('3');
		await page.getByLabel(/Picks per Round/i).fill('8');

		await page.getByRole('button', { name: /Create Draft/i }).click();

		// Should redirect to draft detail page
		await expect(page).toHaveURL(/\/drafts\/[a-f0-9-]+/, { timeout: 10000 });

		// Should show the draft details
		await expect(page.getByText(`${uniqueYear} NFL Draft`)).toBeVisible();
	});
});

test.describe('Draft Detail Page', () => {
	test.beforeEach(async ({ page }) => {
		// Create a draft to test with
		await page.goto('/drafts/new');
		await page.getByLabel(/Draft Year/i).fill('2028');
		await page.getByLabel(/Number of Rounds/i).fill('2');
		await page.getByLabel(/Picks per Round/i).fill('4');
		await page.getByRole('button', { name: /Create Draft/i }).click();

		// Wait for redirect to detail page
		await expect(page).toHaveURL(/\/drafts\/[a-f0-9-]+/, { timeout: 10000 });
	});

	test('should display draft header information', async ({ page }) => {
		await expect(page.getByText('2028 NFL Draft')).toBeVisible();
		await expect(page.getByText('NotStarted')).toBeVisible();
	});

	test('should display draft details grid', async ({ page }) => {
		await expect(page.getByText('Rounds')).toBeVisible();
		await expect(page.getByText('Picks per Round')).toBeVisible();
		await expect(page.getByText('Total Picks')).toBeVisible();
	});

	test('should show Initialize Draft Picks button when no picks', async ({ page }) => {
		await expect(page.getByRole('button', { name: /Initialize Draft Picks/i })).toBeVisible();
	});

	test('should initialize picks successfully', async ({ page }) => {
		// Click initialize button
		await page.getByRole('button', { name: /Initialize Draft Picks/i }).click();

		// Wait for picks to load
		await expect(page.getByText(/Round 1/i)).toBeVisible({ timeout: 10000 });

		// Should show draft board with picks
		await expect(page.getByText('Draft Board')).toBeVisible();
	});

	test('should show draft progress after initialization', async ({ page }) => {
		// Initialize picks
		await page.getByRole('button', { name: /Initialize Draft Picks/i }).click();

		// Wait for initialization
		await expect(page.getByText(/Round 1/i)).toBeVisible({ timeout: 10000 });

		// Should show progress section
		await expect(page.getByText('Draft Progress')).toBeVisible();

		// Should show 0 picks made (initialized but not drafted)
		await expect(page.getByText(/0 \/ \d+ picks made/i)).toBeVisible();
	});

	test('should display Start Draft button', async ({ page }) => {
		await expect(page.getByRole('button', { name: /Start Draft/i })).toBeVisible();
	});

	test('should navigate back to drafts list', async ({ page }) => {
		await page.getByRole('button', { name: /Back to Drafts/i }).click();

		await expect(page).toHaveURL('/drafts');
	});
});
