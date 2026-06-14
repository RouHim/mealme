import { expect, type APIRequestContext, type Page } from '@playwright/test';

const pagesWithInit = new WeakSet<Page>();

export async function setLocale(page: Page, locale: 'en' | 'de'): Promise<void> {
	if (pagesWithInit.has(page)) return;
	await page.addInitScript((l) => {
		if (!localStorage.getItem('mealme.locale')) {
			localStorage.setItem('mealme.locale', l);
		}
	}, locale);
}

export async function gotoEnglish(page: Page): Promise<void> {
	await setLocale(page, 'en');
	await page.goto('/');
}

export async function resetMeals(request: APIRequestContext): Promise<void> {
	const res = await request.get('/api/meals');
	if (!res.ok()) return;
	const meals = (await res.json()) as Array<{ id: number }>;
	await Promise.all(meals.map((m) => request.delete(`/api/meals/${m.id}`)));
}

export async function createMeal(
	page: Page,
	name: string,
	ingredients: Array<{ name: string; quantity?: string }>
): Promise<void> {
	// Meal name — use exact role match to avoid ambiguity with ingredient name fields
	await page.getByRole('textbox', { name: 'Name', exact: true }).fill(name);
	// Fill each ingredient row
	for (let i = 0; i < ingredients.length; i++) {
		const ing = ingredients[i];
		if (i > 0) {
			await page.getByRole('button', { name: /^Add ingredient$|^Zutat hinzufügen$/ }).click();
		}
		const rowLabel = `Ingredient name ${i + 1}`;
		await page.getByRole('textbox', { name: rowLabel }).fill(ing.name);
		if (ing.quantity) {
			// Quantity is the second textbox in the same row
			await page.getByPlaceholder('Quantity').nth(i).fill(ing.quantity);
		}
	}
	await page.getByRole('button', { name: /^(Add|Hinzufügen)$/ }).click();
	await expect(page.getByRole('listitem').filter({ hasText: name })).toBeVisible();
}
