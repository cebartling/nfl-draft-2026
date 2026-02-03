import { test, expect } from '@playwright/test';

test.describe('Home Page', () => {
	test('should load home page successfully', async ({ page }) => {
		await page.goto('/');

		// Wait for page to load
		await expect(page).toHaveTitle(/NFL Draft Simulator/i);
	});

	test('should display welcome message', async ({ page }) => {
		await page.goto('/');

		// Check for welcome heading
		await expect(page.locator('h1')).toContainText(/NFL Draft Simulator/i);
	});

	test('should display navigation links', async ({ page }) => {
		await page.goto('/');

		// Check navigation is present
		const nav = page.locator('nav');
		await expect(nav).toBeVisible();

		// Check for navigation links
		await expect(page.getByRole('link', { name: /teams/i })).toBeVisible();
		await expect(page.getByRole('link', { name: /players/i })).toBeVisible();
		await expect(page.getByRole('link', { name: /drafts/i })).toBeVisible();
	});

	test('should navigate to Teams page', async ({ page }) => {
		await page.goto('/');

		// Click Teams link
		await page.getByRole('link', { name: /teams/i }).click();

		// Verify navigation
		await expect(page).toHaveURL(/\/teams/);
	});

	test('should navigate to Players page', async ({ page }) => {
		await page.goto('/');

		// Click Players link
		await page.getByRole('link', { name: /players/i }).click();

		// Verify navigation
		await expect(page).toHaveURL(/\/players/);
	});

	test('should navigate to Drafts page', async ({ page }) => {
		await page.goto('/');

		// Click Drafts link
		await page.getByRole('link', { name: /drafts/i }).click();

		// Verify navigation
		await expect(page).toHaveURL(/\/drafts/);
	});

	test('should have responsive design', async ({ page }) => {
		// Test desktop viewport
		await page.setViewportSize({ width: 1920, height: 1080 });
		await page.goto('/');
		await expect(page.locator('nav')).toBeVisible();

		// Test mobile viewport
		await page.setViewportSize({ width: 375, height: 667 });
		await page.goto('/');
		// Mobile menu might be different
		await expect(page.locator('body')).toBeVisible();
	});
});
