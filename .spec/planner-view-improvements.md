# Feature Specification: Planner View Improvements

**Created**: 2026-06-14
**Status**: Approved
**Input**: Improve the planner view: gray out (but clickable) past weeks, default meal count should be 3 not 5, fix "not found" error when clicking a future week, remove the language toggle and derive locale from browser settings

## Goal

Polish the weekly meal planner with four targeted improvements. Past weeks receive a muted visual treatment so users can instantly distinguish them from current and future weeks, while remaining navigable. The default meal count when generating a new plan drops from 5 to 3. Clicking any week that has no plan — past, present, or future — shows the generate-plan form cleanly instead of a red error banner. The manual language toggle is removed; locale is derived automatically from the browser's language preference.

## User Scenarios

### Scenario 1 - Past weeks appear dimmed but clickable (P1)

User opens the planner and sees a grid of week buttons. Weeks that have already ended (Sunday before today) render with reduced visual prominence — lighter text, lower opacity, or muted background. The user can still click any past week to view or edit its plan.

**Acceptance**
1. Given today is a Wednesday in week 24, when the planner loads, then all weeks 1–23 render with a visually de-emphasized style distinct from weeks 24–53.
2. Given a past week is displayed dimmed, when the user clicks it, then the plan detail panel opens and functions normally (loads existing plan or shows the generate form).
3. Given the user navigates to a past year (e.g., 2025), when the planner renders, then every week in that year renders with the dimmed style.

### Scenario 2 - Default meal count is 3 (P1)

User selects a week with no plan. The generate-plan form appears with the "Number of meals" input pre-filled to 3.

**Acceptance**
1. Given the user selects a week with no existing plan, when the generate form appears, then the meal count input displays "3".
2. Given the meal count defaults to 3, the user can still change it to any value 1–20 before generating.

### Scenario 3 - Future week shows generate form, not error (P1)

User clicks a future week that has never had a plan generated. Instead of seeing a red error banner reading "Not found" (or similar), the generate-plan form appears immediately.

**Acceptance**
1. Given a future week with no plan exists, when the user clicks that week, then the generate-plan form is displayed with no error message.
2. Given a genuine API failure occurs (network error, server 500), when the request fails, then the error banner still appears with an appropriate message.

### Scenario 4 - Language derived from browser (P2)

A new user visits the application. The language toggle button is absent from the header. The UI renders in German if the browser's primary language is German; otherwise, it renders in English.

**Acceptance**
1. Given a browser with `navigator.language` set to `de` or `de-DE`, when the application loads, then the UI renders in German.
2. Given a browser with `navigator.language` set to `en`, `fr`, or any non-German value, when the application loads, then the UI renders in English.
3. Given the language toggle button previously existed in the header, when the application renders, then no language toggle button is visible anywhere in the UI.
4. Given `navigator.language` is unavailable (e.g., SSR context), when the application initializes, then the UI renders in English.

## Functional Requirements

- **FR-001**: Weeks whose ISO Sunday date is strictly before the current local date SHALL render with a visually distinct, reduced-contrast style (e.g., reduced opacity or muted text color) while remaining fully clickable and functional.
- **FR-002**: The current week (the ISO week containing today) SHALL retain its existing highlighted style and SHALL NOT be visually dimmed.
- **FR-003**: The "Number of meals" input field in the plan generate form MUST default to 3 on initial load and when switching to a different week that has no plan.
- **FR-004**: When the backend responds with HTTP 404 for a single-plan fetch (`GET /api/plans?year=X&week=Y`), the planner SHALL treat it as an empty state — displaying the generate-plan form with no error message — identical to the existing behavior before the plan is first loaded.
- **FR-005**: The manual language toggle button (currently an EN/DE toggle in the home page header) SHALL be removed from the application.
- **FR-006**: The application locale SHALL be determined on load solely from `navigator.language`: German (`de`) if the value starts with `de` (case-insensitive), English (`en`) otherwise. No manual override mechanism SHALL remain.
- **FR-007**: The existing `localStorage` key `mealme.locale` SHALL no longer be read or written by the application.

## Edge Cases

- **Year boundary around New Year**: ISO week 1 may start in the previous calendar year and ISO week 52/53 may end in the next calendar year. Past/future determination uses the week's Sunday date, which correctly handles these straddling weeks.
- **Navigating to a past year**: When the user switches to any previous year (e.g., 2025 in 2026), every week in that year's grid renders in the dimmed style.
- **Navigating to a future year**: When the user switches to a future year, every week renders in the default (non-dimmed) style.
- **Network failure vs. missing plan**: The frontend must distinguish between a 404 from `getPlan` (no plan exists — show generate form) and other errors like network failure or server 500 (show error banner). The existing `__REQUEST_FAILED__` sentinel in `api.ts` already handles the "response was not JSON" case; the missing-plan path should not trigger `planError`.
- **SSR / `navigator` unavailable**: During server-side rendering, `navigator` may be undefined. The existing fallback to `'en'` in `detectInitialLocale()` covers this case.
- **Meal count reset**: When the user clicks a new week, the meal count input must reset to the default of 3, not retain the previous week's value.

## Assumptions

- "Past week" means the week's Sunday date is before today's date in the user's local timezone. The current week (containing today) is not considered past.
- Removing the toggle implies removing all locale-persistence mechanisms (`localStorage` read/write of `mealme.locale`), since without a toggle there is no user-initiated action to store.
- The meal count minimum (1) and maximum (20) remain unchanged.
- The "not found" bug is a frontend handling issue: the backend returning 404 for a missing plan is correct REST semantics; the frontend should treat it as a normal empty state rather than surfacing an error message.
- ISO week numbering (Monday–Sunday, week 1 contains the first Thursday) is already the project's week model and remains unchanged.

## Success Criteria

- **SC-001**: All weeks whose Sunday is before today render with a visibly reduced contrast compared to current and future weeks, and clicking any past week navigates to its detail panel without errors.
- **SC-002**: The meal count input in the generate form shows "3" as its initial value when no plan exists.
- **SC-003**: Clicking any week (past, current, or future) that has no plan displays the generate-plan form with zero error messages.
- **SC-004**: No language toggle element exists in the rendered application.
- **SC-005**: A browser with `navigator.language = 'de-DE'` sees the UI in German on first visit; a browser with `navigator.language = 'en-US'` sees the UI in English.
- **SC-006**: The `localStorage` key `mealme.locale` is neither read on startup nor written on any user action.
