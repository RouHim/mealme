# Feature Specification: Planner UI — Button Labels and Week Color Differentiation

**Created**: 2026-06-15
**Status**: Approved
**Input**: On planner view, rename Delete button to "Delete Plan", rename "Add to plan" to "Add a meal", and improve color differentiation between the currently selected week and the current calendar week.

## Goal
Three targeted UX fixes to the meal planner view. First, the destructive "Delete Plan" button label must unambiguously communicate its scope instead of reusing a generic "Delete" label shared with single-meal removal. Second, the action button that adds meals to a plan reads more naturally as "Add a meal" than the current "Add to plan". Third, the week grid must visually distinguish the current calendar week from the user-selected week — today both use the same orange primary color and differ only by a barely-perceptible background tint, making it difficult to tell which week is "now" vs "selected".

## User Scenarios
### Scenario 1 — Delete an entire meal plan (P1)
A user opens an existing plan and decides to delete it entirely. The delete button at the bottom of the plan detail panel clearly reads "Delete Plan" so there is no confusion with the per-meal remove buttons (trash icons). The confirmation dialog reinforces the scope.

**Acceptance**
1. Given a plan is loaded, when the user views the plan detail panel, then a button labeled "Delete Plan" (EN) / "Plan löschen" (DE) is visible at the bottom.
2. Given the "Delete Plan" button is visible, when the user clicks it, then a confirmation dialog appears referencing the plan (e.g., "Delete this plan?").
3. Given the confirmation dialog, when the user confirms, then the plan is deleted and the detail panel closes.

### Scenario 2 — Add a meal to a plan (P1)
A user wants to add meals from their collection into a plan. The button that opens the meal picker overlay reads "Add a meal".

**Acceptance**
1. Given a plan is loaded with at least one meal slot, when the user views the plan detail panel, then a button labeled "Add a meal" (EN) / "Mahlzeit hinzufügen" (DE) is visible above the ingredient summary.
2. Given the meal picker overlay is open, when the user sees a meal in the search results, then each result row has an action button labeled "Add a meal" (consistent with the opener).

### Scenario 3 — Scan the week grid for the current week (P1)
A user opens the planner and scans the week grid to find the current calendar week. The current week is marked with a green accent border, visually distinct from the orange border of the selected week. The user can tell at a glance which week is "this week" and which week they have selected.

**Acceptance**
1. Given the planner loads with no week selected, when the user views the week grid, then the current calendar week cell has a green (`--color-accent`) border and all other unselected weeks have the default neutral border.
2. Given the user clicks on a non-current week, when the selection updates, then the clicked cell has an orange (`--color-primary`) border and background, while the current week cell retains its green border.
3. Given the user clicks on the current week itself, when both `.week-cell--current` and `.week-cell--active` classes apply, then both the green current-week indicator and the orange selected-state indicator are discernible simultaneously (e.g., green border + orange background).

## Functional Requirements
- **FR-001**: A new i18n key `plannerDeletePlan` SHALL be introduced with English value "Delete Plan" and German value "Plan löschen". The planner delete button in `web/src/routes/planner/+page.svelte` SHALL use this key instead of the generic `buttonDelete`.
- **FR-002**: The existing `plannerAddMeal` i18n key SHALL change its English value from "Add to plan" to "Add a meal" and its German value from "Zum Plan hinzufügen" to "Mahlzeit hinzufügen".
- **FR-003**: The CSS rule `.week-cell--current` SHALL use `var(--color-accent)` for its `border-color` instead of `var(--color-primary)`. The `.week-cell--active` rule SHALL continue using `var(--color-primary)` for its border and background. No new CSS variables SHALL be introduced.
- **FR-004**: When `.week-cell--current` and `.week-cell--active` apply to the same cell simultaneously, both indicators SHALL remain discernible. The green accent border SHALL remain visible alongside the orange active background.
- **FR-005**: The `plannerAddMeal` key SHALL be added to the `TranslationKey` union type in `web/src/lib/i18n/types.ts` if not already present.

## Edge Cases
- The meal picker overlay's per-meal action buttons also use the `plannerAddMeal` i18n key. With the new translation "Add a meal", these buttons read naturally in-context ("Add a meal" next to each search result). This is intentional, not a regression.
- The delete confirmation dialog in `onDeletePlan` already reads "Delete this plan?" — no change needed there.
- E2E tests that locate elements by `.week-cell--current` or `.week-cell--active` CSS classes remain valid since class names are unchanged; only CSS property values differ. Tests that assert specific button text using the old labels will need updating.
- Color contrast: the green accent color (`#65a30d`) on the warm background (`#fff8f0`) must meet WCAG AA contrast for UI components (3:1 minimum). If it falls short, a darker green shade or thicker border is an acceptable adjustment within this requirement.

## Assumptions
- The existing `--color-accent` CSS variable (`#65a30d`, green) is the appropriate shade for the current-week indicator and no new variable is introduced.
- The meal picker per-item button sharing the `plannerAddMeal` key is acceptable — "Add a meal" reads naturally in both the "add meal to plan" opener context and the per-result action context.
- E2E tests that reference button labels or CSS class names will be updated as part of implementation; this spec does not enumerate every affected test.

## Success Criteria
- **SC-001**: The planner delete button displays "Delete Plan" (EN) / "Plan löschen" (DE) and is visually and semantically distinct from per-meal remove buttons.
- **SC-002**: The meal-picker opener button displays "Add a meal" (EN) / "Mahlzeit hinzufügen" (DE).
- **SC-003**: In the week grid, the current calendar week cell has a green border and the selected week cell has an orange border; the two states are distinguishable at a glance by any user with normal color vision.
- **SC-004**: When the current week is also the selected week, both the green current-week indicator and the orange selected-state indicator are simultaneously visible.
- **SC-005**: All existing planner E2E tests pass after the changes, modulo label text updates.
