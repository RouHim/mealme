# Feature Specification: Feather Icons Integration

**Created**: 2026-06-14
**Status**: Approved
**Input**: Use Feather icons (embedded) and use icons in the UI where it makes sense

## Goal
Adopt Feather icons as the project's single icon system using manually inlined SVGs — zero additional npm dependencies, no CDN, no runtime network requests. Establish clear conventions for when and how to use icons, and apply them to UI elements where they improve clarity: action buttons, navigation controls, empty/error states, and form controls. Remove dead icon code and fix misused icons.

## User Scenarios

### Scenario 1 - Icon-augmented action buttons (P1)
A user viewing their meal list can quickly distinguish edit from delete actions on each card because buttons carry both a recognizable icon (pencil for edit, trash for delete) and a text label. The icons improve scanability without replacing text — the text remains the primary affordance.

**Acceptance**
1. Given a meal card in the list, When rendered, Then the edit button displays a pencil icon alongside the "Edit" text and the delete button displays a trash icon alongside the "Delete" text
2. Given a meal card, When the user interacts with either button, Then the icon and text behave as one clickable unit with no layout shift
3. Given the form card, When in add mode, Then the submit button displays a plus icon; When in edit mode, Then it displays a check icon alongside the "Save" text
4. Given the form card in edit mode, When the cancel button is rendered, Then it displays an X icon alongside the "Cancel" text

### Scenario 2 - Error and empty states (P1)
A user encountering an error or empty state sees a contextual icon that reinforces the message — an alert icon for errors, a meal illustration for the empty database, and a search icon for no results.

**Acceptance**
1. Given a load or save error, When the error banner is displayed, Then an alert icon appears inline with the error message
2. Given an empty database with no active search, When the page loads, Then the empty-state illustration icon is displayed above the guidance text
3. Given an active search that returns zero meals, When the results area renders, Then a search icon is displayed above the "no results" message

### Scenario 3 - Developer adding a new icon (P2)
A developer needs to add a new Feather icon to the UI. They copy the SVG markup from the Feather icons set, add it as a new branch in `Icon.svelte`, update the `IconName` type union, and use `<Icon name="...">` in the template. TypeScript catches any typo in the icon name at compile time.

**Acceptance**
1. Given a developer adds a new icon name to the `IconName` type but forgets the SVG branch, Then using that name renders nothing visible (no crash, no console error)
2. Given a developer uses an icon name not in the `IconName` union, Then TypeScript reports a compile error
3. Given a developer adds a custom (non-Feather) icon, Then it is clearly commented as a custom exception in `Icon.svelte`

## Functional Requirements

- **FR-001**: All icons must be embedded as inline SVG markup within `Icon.svelte` — no CDN references, no `<img>` external sources, no runtime `fetch` of icon assets
- **FR-002**: Icons must use `stroke="currentColor"` and `fill="none"` so they inherit text color from the parent element's CSS `color` property, making them compatible with any theme (light/dark) without per-icon color overrides
- **FR-003**: Decorative icons (those accompanying text labels) must have `aria-hidden="true"` on the `<svg>` element. Interactive elements where the icon is the sole visible content must have an `aria-label` on the parent interactive element (button, link)
- **FR-004**: The `Icon.svelte` component must accept props: `name` (required, of type `IconName`), `size` (optional `number`, default 24), `class` (optional `string`, default empty)
- **FR-005**: The set of available icon names must be defined as a TypeScript string union type `IconName` exported from or co-located with the component. Using an undefined name must produce a TypeScript compile error
- **FR-006**: Custom icons not originating from the Feather icon set are permitted but must be documented with an inline comment in `Icon.svelte` marking them as custom exceptions (e.g., `<!-- custom: not from Feather -->`)
- **FR-007**: Icons defined in `Icon.svelte` but with zero usages across the codebase must be removed (YAGNI). Specifically, the `plate` icon must be removed
- **FR-008**: The planner's close-picker button must use an X/close icon instead of the currently misused `alert` icon
- **FR-009**: The following UI elements must display an icon alongside their text label: edit buttons (pencil), delete buttons on meal cards (trash), the form submit button (plus in add mode, check in edit mode), the form cancel button (X)
- **FR-010**: The "no results" empty state (when a search yields zero meals) must display a search icon above the message text
- **FR-011**: The navigation link to the planner page must display a calendar icon

## Key Entities

- **IconName**: A TypeScript union type enumerating all valid icon names. Each name maps to exactly one SVG markup branch in `Icon.svelte`. Example values: `'search'`, `'alert'`, `'trash'`, `'plus'`, `'x'`, `'edit-3'`, `'check'`, `'calendar'`, `'chevron-left'`, `'chevron-right'`, `'empty-meals'`
- **Icon component** (`Icon.svelte`): A single Svelte 5 component that renders an inline `<svg>` element based on the `name` prop. All Feather-sourced icons share a 24x24 viewBox with stroke-based rendering. Custom icons may deviate from the 24x24 viewBox but must set an explicit default size

## Edge Cases

- Icons rendered at sizes ≤14px must remain visually legible. This is relevant for the calendar badge icon (12px) and trash icon on ingredient rows (14-16px)
- Icons placed inline with text in buttons must be vertically aligned with the text baseline — no misalignment or layout shift on render
- The reduced-motion media query has no impact on icons (they do not animate), so no special handling is required
- When an icon name is valid in the type union but its SVG branch is accidentally missing, the component must render nothing (empty output) rather than throwing a runtime error or displaying broken markup
- Custom icons with non-24x24 viewBoxes must not be stretched when rendered at their default size — they must use their own viewBox dimensions as the default size

## Research Notes

- Feather icons v4.29.2, MIT license, 287 icons, ~176K weekly npm downloads. The npm package (`feather-icons`) depends on `classnames` and `core-js` — neither is needed when manually inlining SVG markup
- The Feather GitHub repository (github.com/feathericons/feather) has a slow maintenance cadence but the 287-icon set is stable, widely adopted, and design-complete — no churn risk for an inlined subset
- Common CRUD icon conventions: pencil for edit, trash for delete, check/plus for save/add, X for close/cancel, magnifying glass for search. These are near-universal in web applications and improve list scanability without replacing text labels

## Assumptions

- Icons are always monochrome, inheriting color from the parent element's text color via `currentColor`
- Default icon size is 24x24 pixels (matches Feather's native grid) unless explicitly overridden by the `size` prop
- All Feather-sourced icons use `stroke-width="2"`, `stroke-linecap="round"`, and `stroke-linejoin="round"` — matching Feather's design parameters
- No new npm dependencies are introduced — SVG markup is copied directly from the Feather icon set into `Icon.svelte`
- The `empty-meals` custom icon remains as a documented exception within `Icon.svelte`; it is not replaceable by any existing Feather icon
- The icon placements enumerated in FR-009 through FR-011 represent the initial set determined to "make sense"; additional icons may be added later following the same conventions and the process in Scenario 3
- The `Icon.svelte` component is the single canonical source for all icons in the application — no alternative icon components or scattered inline SVGs exist

## Success Criteria

- **SC-001**: Every icon rendered in the UI originates from `Icon.svelte` — no raw `<svg>` elements exist in page templates
- **SC-002**: The `IconName` type union contains exactly the set of icons that have at least one usage in the codebase — no dead icon definitions
- **SC-003**: All buttons with icons pass accessibility checks: decorative SVGs have `aria-hidden="true"`, icon-only buttons have `aria-label`
- **SC-004**: The planner close-picker button renders an X icon, not an alert icon
- **SC-005**: The application builds with zero additional npm dependencies beyond those already present before this feature
- **SC-006**: TypeScript type-checking (`npm run check` in `web/`) passes with no errors related to icon names
- **SC-007**: All existing tests (Vitest frontend tests, Playwright E2E tests) continue to pass after icon changes
