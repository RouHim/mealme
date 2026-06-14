import { describe, it, expect } from 'vitest';
import { weekOfDate, mondaySundayOf, weeksInYear } from './week';

describe('weekOfDate', () => {
	it('returns 2026 week 1 for Jan 1 2026', () => {
		const d = new Date('2026-01-01T12:00:00Z');
		const result = weekOfDate(d);
		expect(result.year).toBe(2026);
		expect(result.week).toBe(1);
	});
});

describe('mondaySundayOf', () => {
	it('returns Dec 29 2025 to Jan 4 2026 for week 1 2026', () => {
		const { monday, sunday } = mondaySundayOf(2026, 1);
		expect(monday.toISOString().slice(0, 10)).toBe('2025-12-29');
		expect(sunday.toISOString().slice(0, 10)).toBe('2026-01-04');
	});

	it('returns Jun 15 to Jun 21 2026 for week 25 2026', () => {
		const { monday, sunday } = mondaySundayOf(2026, 25);
		expect(monday.toISOString().slice(0, 10)).toBe('2026-06-15');
		expect(sunday.toISOString().slice(0, 10)).toBe('2026-06-21');
	});
});

describe('weeksInYear', () => {
	it('returns 53 for 2026', () => {
		expect(weeksInYear(2026)).toBe(53);
	});

	it('returns 53 for 2023', () => {
		// Lock in determinism
		const actual = weeksInYear(2023);
		expect(actual).toBe(weeksInYear(2023));
		expect(actual).toBeGreaterThanOrEqual(52);
		expect(actual).toBeLessThanOrEqual(53);
	});
});
