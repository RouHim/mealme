import { describe, it, expect, vi, beforeEach } from 'vitest';
import { listMeals, createMeal, updateMeal, deleteMeal, mealImageUrl, listPlansForYear, getPlan, createPlan, updatePlan, deletePlan } from './api';
import type { Meal, MealPayload, Plan, PlanSummaryItem, NewPlanRequest, PlanPatch } from './types';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch;

beforeEach(() => {
	mockFetch.mockReset();
});

function mockResponse(status: number, body?: unknown) {
	const init: ResponseInit = { status };
	if (body !== undefined) {
		mockFetch.mockResolvedValueOnce({
			ok: status >= 200 && status < 300,
			status,
			json: async () => body,
		} satisfies Partial<Response>);
	} else {
		mockFetch.mockResolvedValueOnce({
			ok: status >= 200 && status < 300,
			status,
		} satisfies Partial<Response>);
	}
}

// ---------------------------------------------------------------------------
// Meal API
// ---------------------------------------------------------------------------

describe('listMeals', () => {
	it('calls /api/meals without search', async () => {
		mockResponse(200, []);
		await listMeals();
		expect(mockFetch).toHaveBeenCalledWith('/api/meals', undefined);
	});

	it('calls /api/meals?search=...', async () => {
		mockResponse(200, []);
		await listMeals('pizza');
		expect(mockFetch).toHaveBeenCalledWith('/api/meals?search=pizza', undefined);
	});
});

describe('createMeal', () => {
	it('sends multipart form with name and ingredients', async () => {
		const payload: MealPayload = { name: 'Test', ingredients: [{ name: 'stuff', quantity: null }] };
		const mealResponse: Meal = { id: 1, name: 'Test', ingredients: [{ name: 'stuff', quantity: null }], last_planned_at: null, created_at: '', updated_at: '', has_image: false };
		mockResponse(201, mealResponse);
		await createMeal(payload);
		expect(mockFetch).toHaveBeenCalledTimes(1);
		const [url, opts] = mockFetch.mock.calls[0];
		expect(url).toBe('/api/meals');
		expect(opts.method).toBe('POST');
		expect(opts.body).toBeInstanceOf(FormData);
		const fd = opts.body as FormData;
		expect(fd.get('name')).toBe('Test');
		expect(fd.get('ingredients')).toBe(JSON.stringify(payload.ingredients));
		expect(fd.get('image')).toBeNull();
		// Browser sets multipart boundary — no explicit content-type header
		expect(opts.headers).toBeUndefined();
	});

	it('includes image file when provided', async () => {
		const payload: MealPayload = { name: 'Pizza', ingredients: [{ name: 'cheese', quantity: null }] };
		const mealResponse: Meal = { id: 2, name: 'Pizza', ingredients: [{ name: 'cheese', quantity: null }], last_planned_at: null, created_at: '', updated_at: '', has_image: true };
		mockResponse(201, mealResponse);
		const file = new File([new Uint8Array([1, 2, 3])], 'photo.png', { type: 'image/png' });
		await createMeal(payload, file);
		const fd = mockFetch.mock.calls[0][1].body as FormData;
		expect(fd.get('image')).toBeInstanceOf(File);
		expect((fd.get('image') as File).name).toBe('photo.png');
	});

	it('handles null image gracefully', async () => {
		const payload: MealPayload = { name: 'X', ingredients: [{ name: 'y', quantity: null }] };
		const mealResponse: Meal = { id: 3, name: 'X', ingredients: [{ name: 'y', quantity: null }], last_planned_at: null, created_at: '', updated_at: '', has_image: false };
		mockResponse(201, mealResponse);
		await createMeal(payload, null);
		const fd = mockFetch.mock.calls[0][1].body as FormData;
		expect(fd.get('image')).toBeNull();
	});
});

describe('updateMeal', () => {
	it('sends multipart form with name and ingredients', async () => {
		const payload: MealPayload = { name: 'Updated', ingredients: [{ name: 'new', quantity: null }] };
		const mealResponse: Meal = { id: 3, name: 'Updated', ingredients: [{ name: 'new', quantity: null }], last_planned_at: null, created_at: '', updated_at: '', has_image: false };
		mockResponse(200, mealResponse);
		await updateMeal(3, payload);
		const [url, opts] = mockFetch.mock.calls[0];
		expect(url).toBe('/api/meals/3');
		expect(opts.method).toBe('PUT');
		const fd = opts.body as FormData;
		expect(fd.get('name')).toBe('Updated');
		expect(fd.get('ingredients')).toBe(JSON.stringify(payload.ingredients));
		expect(fd.get('image_action')).toBeNull();
	});

	it('sends image_action=remove when removing', async () => {
		const payload: MealPayload = { name: 'X', ingredients: [{ name: 'y', quantity: null }] };
		const mealResponse: Meal = { id: 4, name: 'X', ingredients: [{ name: 'y', quantity: null }], last_planned_at: null, created_at: '', updated_at: '', has_image: false };
		mockResponse(200, mealResponse);
		await updateMeal(4, payload, { removeImage: true });
		const fd = mockFetch.mock.calls[0][1].body as FormData;
		expect(fd.get('image_action')).toBe('remove');
	});

	it('sends image file when replacing', async () => {
		const payload: MealPayload = { name: 'X', ingredients: [{ name: 'y', quantity: null }] };
		const mealResponse: Meal = { id: 5, name: 'X', ingredients: [{ name: 'y', quantity: null }], last_planned_at: null, created_at: '', updated_at: '', has_image: true };
		mockResponse(200, mealResponse);
		const file = new File([new Uint8Array([4, 5, 6])], 'new.jpg', { type: 'image/jpeg' });
		await updateMeal(5, payload, { image: file });
		const fd = mockFetch.mock.calls[0][1].body as FormData;
		expect(fd.get('image')).toBeInstanceOf(File);
		expect((fd.get('image') as File).name).toBe('new.jpg');
	});
});

describe('deleteMeal', () => {
	it('deletes /api/meals/:id', async () => {
		mockResponse(204);
		await deleteMeal(7);
		expect(mockFetch).toHaveBeenCalledWith('/api/meals/7', { method: 'DELETE' });
	});
});

describe('mealImageUrl', () => {
	it('returns the correct image endpoint URL', () => {
		expect(mealImageUrl(42)).toBe('/api/meals/42/image');
	});
});

describe('error handling', () => {
	it('extracts server error message from JSON body', async () => {
		mockResponse(400, { error: 'name must not be empty' });
		await expect(listMeals()).rejects.toThrow('name must not be empty');
	});
});

// ---------------------------------------------------------------------------
// Plan API
// ---------------------------------------------------------------------------

describe('listPlansForYear', () => {
	it('calls /api/plans?year=...', async () => {
		mockResponse(200, []);
		await listPlansForYear(2026);
		expect(mockFetch).toHaveBeenCalledWith('/api/plans?year=2026', undefined);
	});

	it('throws on non-array response', async () => {
		mockResponse(200, { not: 'array' });
		await expect(listPlansForYear(2026)).rejects.toThrow('expected array');
	});
});

describe('getPlan', () => {
	it('calls /api/plans?year=...&week=...', async () => {
		const plan: Plan = { id: 1, year: 2026, week_number: 1, created_at: '', meals: [], ingredient_summary: [] };
		mockResponse(200, plan);
		const result = await getPlan(2026, 1);
		expect(mockFetch).toHaveBeenCalledWith('/api/plans?year=2026&week=1');
		expect(result).toEqual(plan);
	});

	it('returns null on 404', async () => {
		mockResponse(404);
		const result = await getPlan(2026, 53);
		expect(result).toBeNull();
	});
});

describe('createPlan', () => {
	it('posts JSON body', async () => {
		const payload: NewPlanRequest = { year: 2026, week_number: 1, meal_count: 3 };
		const plan: Plan = { id: 1, year: 2026, week_number: 1, created_at: '', meals: [], ingredient_summary: [] };
		mockResponse(201, plan);
		await createPlan(payload);
		expect(mockFetch).toHaveBeenCalledWith('/api/plans', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(payload)
		});
	});
});

describe('updatePlan', () => {
	it('puts JSON body', async () => {
		const payload: PlanPatch = { meal_ids: [1, 2] };
		const plan: Plan = { id: 1, year: 2026, week_number: 1, created_at: '', meals: [], ingredient_summary: [] };
		mockResponse(200, plan);
		await updatePlan(2026, 1, payload);
		expect(mockFetch).toHaveBeenCalledWith('/api/plans/2026/1', {
			method: 'PUT',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(payload)
		});
	});
});

describe('deletePlan', () => {
	it('deletes /api/plans/:year/:week', async () => {
		mockResponse(204);
		await deletePlan(2026, 1);
		expect(mockFetch).toHaveBeenCalledWith('/api/plans/2026/1', { method: 'DELETE' });
	});
});
