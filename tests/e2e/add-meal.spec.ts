import { test, expect } from '@playwright/test';
import { createMeal, resetMeals, setLocale } from './_helpers';

test.describe('Add meal', () => {
	test.beforeEach(async ({ request, page }) => {
		await setLocale(page, 'en');
		await resetMeals(request);
	});

	test('adds a valid meal and shows it in the list', async ({ page }) => {
		await page.goto('/');
		await createMeal(page, 'Salad', [{ name: 'lettuce' }, { name: 'tomato' }]);
		await expect(page.getByRole('listitem').filter({ hasText: 'Salad' })).toBeVisible();
	});

	test('shows validation error for empty name', async ({ page }) => {
		await page.goto('/');
		// Fill ingredient name but leave meal name empty
		await page.getByRole('textbox', { name: 'Ingredient name 1' }).fill('x');
		// Click the form submit button (exact match to avoid "Add ingredient" button)
		await page.getByRole('button', { name: 'Add', exact: true }).click();
		await expect(page.getByText('Name is required')).toBeVisible();
	});

	test('shows validation error for empty ingredients', async ({ page }) => {
		await page.goto('/');
		await page.getByRole('textbox', { name: 'Name', exact: true }).fill('x');
		// Leave ingredient row empty, click submit
		await page.getByRole('button', { name: 'Add', exact: true }).click();
		await expect(page.getByText('At least one ingredient is required')).toBeVisible();
	});
});
