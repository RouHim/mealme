# Feature Specification: Seed Meals CLI Subcommand

**Created**: 2026-06-14
**Status**: Approved
**Input**: Add a `seed` CLI subcommand that populates the database with 15 deterministic random meals for testing and development.

## Goal

Give developers and testers a fast, reproducible way to populate MealMe with a working dataset of 15 meals without manually entering them through the UI. The `seed` subcommand generates the same 15 meals every invocation (deterministic, seeded RNG), resolves the data directory the same way the server does, and is safe to run repeatedly — it skips if meals already exist. Useful for manual QA, screenshot generation, exploratory testing, and quick evaluation of the application.

## User Scenarios

### Scenario 1 - Seed an empty database (P1)

A developer clones the repository and runs `mealme seed` on a fresh checkout with no existing database. The command creates the data directory, initializes the database schema via migrations, inserts 15 deterministic meals with associated ingredients, prints a success summary, and exits.

**Acceptance**
1. Given no database exists and no `MEALME_DATA_DIR` is set, when the developer runs `mealme seed`, then the default data directory (`data/`) is created, `data/meals.db` is initialized, 15 meals with 2–6 ingredients each are inserted, and the command prints the meal count and exits with status 0.
2. Given the 15 seeded meals were just created, when the developer starts the server (`mealme`) and opens the UI, then all 15 meals are visible in the meal list.

### Scenario 2 - Seed with custom data directory (P1)

A developer sets `MEALME_DATA_DIR=/tmp/test-db` and runs `mealme seed`. The command seeds into `/tmp/test-db/meals.db` instead of the default location.

**Acceptance**
1. Given `MEALME_DATA_DIR=/tmp/test-db` and `/tmp/test-db` does not exist, when the developer runs `mealme seed`, then the directory is created, the database is placed at `/tmp/test-db/meals.db`, and 15 meals are inserted.

### Scenario 3 - Skip when meals already exist (P1)

A developer has already seeded the database (or manually added meals) and runs `mealme seed` again. The command detects that meals are present and skips seeding without inserting duplicates.

**Acceptance**
1. Given the database already contains one or more meals, when the developer runs `mealme seed`, then no new meals are inserted, an informational message is printed indicating seeding was skipped, and the command exits with status 0.

### Scenario 4 - Deterministic output (P2)

Two developers on different machines each run `mealme seed` on a fresh database. Both get the exact same 15 meals with the same names and ingredients.

**Acceptance**
1. Given two fresh databases on different machines, when each runs `mealme seed`, then the resulting meals have identical names and ingredient lists (same meal names, same ingredient names, same quantities).

### Scenario 5 - Error when data directory is unwritable (P2)

A developer runs `mealme seed` with a data directory path they cannot create or write to. The command fails with a clear error message.

**Acceptance**
1. Given `MEALME_DATA_DIR=/root/mealme` and the process lacks write permission to `/root`, when the developer runs `mealme seed`, then the command exits with a non-zero status and prints an error message describing the failure.

## Functional Requirements

- **FR-001**: The application MUST accept a `seed` subcommand as the first positional argument (`mealme seed`).
- **FR-002**: The `seed` subcommand MUST resolve the data directory using the same `MEALME_DATA_DIR` / CWD logic as the server (default: `data/` relative to CWD).
- **FR-003**: When the data directory does not exist, the `seed` subcommand MUST auto-create it (including parent directories) before initializing the database.
- **FR-004**: The `seed` subcommand MUST run database migrations if the database file is new or missing, ensuring the schema is current before inserting data.
- **FR-005**: The `seed` subcommand MUST check whether the `meals` table already contains rows. If any meals exist, seeding MUST be skipped — no new meals are inserted.
- **FR-006**: When seeding proceeds, exactly 15 meals MUST be inserted, each with at least 2 and at most 6 associated ingredients with non-empty quantities.
- **FR-007**: The seeded meals MUST be deterministic — the same 15 meal names, ingredient names, and quantities MUST be produced on every invocation, on any machine, regardless of locale or OS.
- **FR-008**: Each seeded meal's `created_at` and `updated_at` timestamps MUST be set to the current UTC time at insertion.
- **FR-009**: Ingredient names inserted during seeding MUST be normalized via the existing `normalize_ingredient_name` function (lowercased, whitespace-collapsed).
- **FR-010**: On successful seeding, the command MUST print a summary to stdout indicating the number of meals inserted and then exit with status 0.
- **FR-011**: When seeding is skipped (meals already exist), the command MUST print an informational message to stdout indicating the skip and exit with status 0.
- **FR-012**: On any error (unwritable data directory, migration failure, insertion failure), the command MUST print a descriptive error message to stderr and exit with a non-zero status.
- **FR-013**: The `seed` subcommand MUST NOT start the HTTP server.
- **FR-014**: The meal names and ingredient lists used for seeding MUST be embedded in the application binary (no network calls, no file reads beyond the database).

## Edge Cases

- **Empty database with no meals but other tables have rows**: If the database has been initialized (migrations run) but no meals were ever created, seeding proceeds normally — the presence of empty `ingredients` or `week_plans` tables does not block seeding.
- **Repeated invocations after skip**: Running `mealme seed` multiple times on a populated database always skips with the same informational message.
- **Seed on fresh install then server start**: Running `mealme seed` then `mealme` (the server) in sequence works — the server opens the same database and serves the seeded meals.
- **No `seed` subcommand when running the server**: Omitting the subcommand (bare `mealme`) behaves exactly as before — the HTTP server starts normally with no seeding.
- **Unknown subcommand**: Running `mealme foo` (an unrecognized subcommand) should produce a usage error. The addition of `seed` does not change the behavior for unknown arguments.
- **Concurrent access**: If the server is running and using the database while `mealme seed` is invoked against the same database, SQLite's locking handles the conflict. The seed command may fail with a database-locked error, which is acceptable — the developer should stop the server first.

## Assumptions

- The `seed` subcommand is parsed from `std::env::args()` without adding a CLI argument-parsing library (e.g., `clap`). The first positional argument `seed` triggers the subcommand; all other arguments and flags are ignored by this feature.
- The "meals exist" check is a simple `SELECT COUNT(*) FROM meals` — if the count is greater than zero, seeding is skipped.
- Meal names and ingredient lists are embedded as constants in the Rust source code (not loaded from an external file).
- The deterministic RNG uses a fixed seed value hardcoded in the source.
- The `rand` crate (already a dependency) is used for the seeded RNG.
- Ingredient names are lowercased and whitespace-normalized via the existing `normalize_ingredient_name` function in `db.rs`.
- `created_at` and `updated_at` timestamps reflect the actual insertion time, not fabricated historical dates.
- The `seed` subcommand does not require the `--release` build flag; it works in both debug and release builds.
- The feature does not add any new dependencies to `Cargo.toml`.

## Success Criteria

- **SC-001**: Running `mealme seed` on a system with no prior database creates `data/meals.db` with exactly 15 meals, exits 0, and prints a success summary.
- **SC-002**: Running `mealme seed` a second time on the same database prints a skip message, exits 0, and leaves the meal count unchanged at 15.
- **SC-003**: Running `MEALME_DATA_DIR=/tmp/seed-test mealme seed` creates `/tmp/seed-test/meals.db` with 15 meals.
- **SC-004**: Running `mealme seed` on two different machines produces byte-identical meal names and ingredient lists in the database.
- **SC-005**: Running the server after `mealme seed` (`mealme` with no subcommand) serves the 15 seeded meals via `GET /api/meals`.
- **SC-006**: Running `MEALME_DATA_DIR=/root/denied mealme seed` as a non-root user exits non-zero with an error message.
- **SC-007**: Each of the 15 seeded meals has between 2 and 6 ingredients, and every ingredient has a non-empty quantity string.
