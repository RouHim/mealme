export interface IngredientQuantity {
	name: string;
	quantity: string | null;
}

export interface Meal {
	id: number;
	name: string;
	ingredients: IngredientQuantity[];
	last_planned_at: string | null; // ISO 8601 or null
	created_at: string; // ISO 8601
	updated_at: string; // ISO 8601
}

export interface NewIngredientLine {
	name: string;
	quantity: string | null;
}

export interface MealPayload {
	name: string;
	ingredients: NewIngredientLine[];
}

export interface NumericTotal {
	value: number;
	unit: string | null;
}

export interface IngredientSummaryEntry {
	name: string;
	numeric_total: NumericTotal | null;
	non_numeric: string[];
}

export interface Plan {
	id: number;
	year: number;
	week_number: number;
	created_at: string;
	meals: Meal[];
	ingredient_summary: IngredientSummaryEntry[];
}

export interface PlanSummaryItem {
	year: number;
	week_number: number;
	id: number;
	meal_count: number;
}

export interface NewPlanRequest {
	year: number;
	week_number: number;
	meal_count: number;
}

export interface PlanPatch {
	meal_ids: number[];
}
