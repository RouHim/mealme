# MealMe

A single-binary local-first web application for managing a personal collection of meals. Built with Rust (axum + rusqlite) and Svelte 5, all frontend assets embedded in the binary.

## Quickstart

```bash
cargo run --release
```

Then open **http://127.0.0.1:11341** in your browser.

The server listens on `127.0.0.1:11341` and persists data in `./meals.db` (SQLite, auto-created on first run).

## Requirements

- **Rust** 1.85+ (with Cargo)
- **Node.js** 26+ (build-time only — `build.rs` runs `npm install && npm run build` in `web/`)
- **`just`** (optional, for E2E workflow — install via `cargo install just` or your package manager)

No Docker, nginx, or Node.js runtime needed after compilation.

## Development

```bash
# Run all Rust tests (40 tests)
cargo test

# Run all frontend tests (22 tests)
cd web && npm test

# Build release binary
cargo build --release

# Run with debug logging
RUST_LOG=debug cargo run
```

### E2E tests

```bash
# One-time setup
just e2e-install

# Run the suite
just e2e

# Debug locally
just e2e-headed
just e2e-ui
```

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/meals` | List all meals (ordered by most recent) |
| GET | `/api/meals?search=term` | Search meals by name or ingredients (case-insensitive) |
| POST | `/api/meals` | Create a meal `{"name":"...","ingredients":[{"name":"...","quantity":null}]}` |
| GET | `/api/meals/:id` | Get a single meal |
| PUT | `/api/meals/:id` | Update a meal |
| DELETE | `/api/meals/:id` | Delete a meal |
| POST | `/api/plans` | Generate a plan `{"year":2026,"week_number":1,"meal_count":3}` |
| GET | `/api/plans?year=&week=` | Get a specific plan with meals and ingredient summary |
| GET | `/api/plans?year=` | List all plans for a year |
| PUT | `/api/plans/:year/:week` | Update plan meals `{"meal_ids":[1,2,3]}` |
| DELETE | `/api/plans/:year/:week` | Delete a plan |

**Ingredients** are now stored as normalized rows: each meal has one or more ingredients with optional quantity text (e.g. `"200g"`, `"2 cups"`, `"a pinch"`). Plans aggregate ingredients — identical ingredients are merged and numeric quantities summed. The `/planner` route provides a week calendar for generating and managing weekly meal plans.

Validation: meal name (1–200 chars), ingredient name (1–100 chars per line), ingredient quantity (0–50 chars), max 100 ingredient lines. All API paths are under `/api`; everything else serves the SPA frontend.
