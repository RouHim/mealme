import { describe, it, expect } from 'vitest';
import { validateMeal } from '$lib/validation';
import type { NewIngredientLine } from '$lib/types';

function longString(n: number): string {
	return Array.from({ length: n }, () => 'a').join('');
}

function ingr(name: string, quantity?: string): NewIngredientLine {
	return { name, quantity: quantity ?? null };
}

describe('validateMeal (from page level)', () => {
	it('returns ok for valid input', () => {
		expect(validateMeal('Pasta', [ingr('noodles')])).toEqual({ ok: true });
	});

	it('rejects empty name', () => {
		expect(validateMeal('', [ingr('x')])).toMatchObject({ ok: false, field: 'name' });
	});

	it('accepts name at exactly 200 chars', () => {
		expect(validateMeal(longString(200), [ingr('x')])).toEqual({ ok: true });
	});

	it('rejects name at 201 chars', () => {
		expect(validateMeal(longString(201), [ingr('x')])).toMatchObject({ ok: false, field: 'name' });
	});

	it('rejects empty ingredients', () => {
		expect(validateMeal('x', [])).toMatchObject({ ok: false, field: 'ingredients' });
	});

	it('accepts ingredients with valid lines', () => {
		expect(validateMeal('x', [ingr('stuff', '200g')])).toEqual({ ok: true });
	});

	it('rejects too many ingredients', () => {
		const lines = Array.from({ length: 101 }, (_, i) => ingr(`item ${i}`));
		expect(validateMeal('x', lines)).toMatchObject({ ok: false, field: 'ingredients' });
	});
});
