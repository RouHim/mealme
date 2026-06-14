import { describe, it, expect } from 'vitest';
import { validateMeal } from './validation';
import type { NewIngredientLine } from './types';

function longString(n: number): string {
	return Array.from({ length: n }, () => 'a').join('');
}

function ingr(name: string, quantity?: string): NewIngredientLine {
	return { name, quantity: quantity ?? null };
}

describe('validateMeal', () => {
	it('returns ok for valid name and one ingredient line', () => {
		expect(validateMeal('Pasta', [ingr('noodles')])).toEqual({ ok: true });
	});

	it('rejects empty name', () => {
		expect(validateMeal('', [ingr('x')])).toMatchObject({ ok: false, field: 'name' });
	});

	it('rejects whitespace-only name', () => {
		expect(validateMeal('   ', [ingr('x')])).toMatchObject({ ok: false, field: 'name' });
	});

	it('accepts name at exactly 200 chars', () => {
		expect(validateMeal(longString(200), [ingr('x')])).toEqual({ ok: true });
	});

	it('rejects name at 201 chars', () => {
		expect(validateMeal(longString(201), [ingr('x')])).toMatchObject({ ok: false, field: 'name' });
	});

	it('rejects no ingredient lines', () => {
		expect(validateMeal('x', [])).toMatchObject({ ok: false, field: 'ingredients' });
	});

	it('rejects ingredient line with empty name', () => {
		expect(validateMeal('x', [ingr('   ')])).toMatchObject({ ok: false, field: 'ingredients' });
	});

	it('rejects ingredient name above 100 chars', () => {
		expect(validateMeal('x', [ingr(longString(101))])).toMatchObject({ ok: false, field: 'ingredients' });
	});

	it('rejects ingredient quantity above 50 chars', () => {
		expect(validateMeal('x', [ingr('valid', longString(51))])).toMatchObject({ ok: false, field: 'ingredients' });
	});

	it('rejects above 100 ingredient lines', () => {
		const lines = Array.from({ length: 101 }, (_, i) => ingr(`ingredient ${i}`));
		expect(validateMeal('x', lines)).toMatchObject({ ok: false, field: 'ingredients' });
	});
});
