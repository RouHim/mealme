# Feature Specification: Data Directory Environment Variable

**Created**: 2026-06-14
**Status**: Approved
**Input**: Introduce a `MEALME_DATA_DIR` env var for defining a data folder (default: `data/`) that contains all persisted application data on disk (database, future assets, etc.).

## Goal

Give users a single environment variable to control where all MealMe persisted data lives on disk. Today that means the SQLite database; tomorrow it covers any additional files the application writes. The data directory is auto-created on startup so the application works out of the box with zero configuration, and the existing `MEALME_DB_PATH` env var is removed in favor of this unified approach.

## User Scenarios

### Scenario 1 - Default data directory (P1)

A user runs `mealme` with no environment variables set. The application creates a `data/` directory in the current working directory (if absent) and stores the SQLite database at `data/meals.db`. The application starts normally.

**Acceptance**
1. Given no `MEALME_DATA_DIR` is set, when the application starts, then it creates `data/` relative to CWD and places `meals.db` inside it.
2. Given `data/` already exists from a previous run, when the application starts, then it reuses the existing directory and opens the existing database.

### Scenario 2 - Custom data directory via absolute path (P1)

A user wants all MealMe data stored in `/opt/mealme`. They set `MEALME_DATA_DIR=/opt/mealme` and run the binary. The application creates `/opt/mealme` (if absent) and stores the database at `/opt/mealme/meals.db`.

**Acceptance**
1. Given `MEALME_DATA_DIR=/opt/mealme` is set, when the application starts and `/opt/mealme` does not exist, then the directory is created and the database is placed at `/opt/mealme/meals.db`.
2. Given `MEALME_DATA_DIR=/opt/mealme` is set and `/opt/mealme` already exists with a database from a prior run, when the application starts, then it connects to the existing database successfully.

### Scenario 3 - Custom data directory via relative path (P1)

A user sets `MEALME_DATA_DIR=./storage` and runs the binary from `/home/user/app`. The application creates `/home/user/app/storage` and stores the database at `/home/user/app/storage/meals.db`.

**Acceptance**
1. Given `MEALME_DATA_DIR=./storage` and CWD is `/home/user/app`, when the application starts, then the directory is created at `/home/user/app/storage` and the database is placed inside it.

### Scenario 4 - Unwritable data directory (P2)

A user sets `MEALME_DATA_DIR` to a path they cannot create or write to (e.g., `/root/mealme` as a non-root user). The application fails to start with a clear error message.

**Acceptance**
1. Given `MEALME_DATA_DIR=/root/mealme` and the process lacks write permission to `/root`, when the application starts, then it exits with a non-zero status and prints an error message describing the failure.
2. Given `MEALME_DATA_DIR=/tmp/meals.db` (an existing file, not a directory), when the application starts, then it exits with a non-zero status and prints an error message indicating the path is not a directory.

### Scenario 5 - Trailing slash normalization (P2)

A user sets `MEALME_DATA_DIR=data/` (with a trailing slash). The application normalizes this to `data` and proceeds normally.

**Acceptance**
1. Given `MEALME_DATA_DIR=data/`, when the application starts, then the trailing slash is stripped and the directory is created as `data` with the database at `data/meals.db`.

## Functional Requirements

- **FR-001**: The application MUST read the `MEALME_DATA_DIR` environment variable on startup.
- **FR-002**: When `MEALME_DATA_DIR` is unset or empty, the application MUST default to `data/` relative to the process's current working directory at startup.
- **FR-003**: The application MUST auto-create the data directory (including any missing parent directories) on startup if it does not already exist.
- **FR-004**: When the data directory already exists and is a directory, the application MUST reuse it without modification.
- **FR-005**: The application MUST place the SQLite database file (`meals.db`) inside the data directory.
- **FR-006**: The application MUST exit with a non-zero status and a descriptive error message if the data directory cannot be created (e.g., permission denied) or if the path exists but is not a directory.
- **FR-007**: The `MEALME_DB_PATH` environment variable MUST be removed; the database path is no longer independently configurable.
- **FR-008**: The data directory path MUST support both relative paths (resolved against CWD at startup) and absolute paths.
- **FR-009**: Trailing path separators in the env var value MUST be normalized (stripped) before use.

## Edge Cases

- **Path exists as a file**: If `MEALME_DATA_DIR` points to an existing file (not a directory), the application must fail with a clear error — it must not overwrite or attempt to use the file as a directory.
- **Relative path with CWD change**: The data directory path is resolved once at startup against the process CWD. Subsequent CWD changes (via `chdir`) do not affect data file locations.
- **Empty string env var**: An explicitly empty `MEALME_DATA_DIR=""` is treated the same as unset — falls back to the `data/` default.
- **Symlinks**: The application resolves the path as given; it does not follow or dereference symlinks for existence checks beyond what the OS filesystem APIs provide.
- **Concurrent processes**: Two `mealme` processes using the same data directory will share the same SQLite database. SQLite's built-in locking handles this; no application-level coordination is required.

## Assumptions

- The default data directory name is `data/` (relative to CWD).
- Auto-creation includes intermediate parent directories (equivalent to `mkdir -p` semantics).
- Both relative and absolute paths are supported; relative paths resolve against the process CWD at startup time.
- The existing `MEALME_DB_PATH` env var is removed entirely — no deprecation period, no fallback chain.
- An unwritable or uncreatable data directory is a fatal startup error; there is no fallback to a temporary directory or alternative location.
- The `meals.db` filename inside the data directory is hardcoded and not configurable through this feature.
- The `data/` directory should be added to `.gitignore` as a project maintenance follow-up (not part of this feature specification).

## Success Criteria

- **SC-001**: Running `mealme` with no env vars creates `data/meals.db` and starts successfully.
- **SC-002**: Running `MEALME_DATA_DIR=/tmp/test-mealme-data mealme` creates `/tmp/test-mealme-data/meals.db` and starts successfully.
- **SC-003**: Running `MEALME_DATA_DIR=/root/denied mealme` as a non-root user exits with a non-zero status and prints an error message containing the path.
- **SC-004**: Setting the (now-removed) `MEALME_DB_PATH` env var has no effect on database location — the database is always at `<data_dir>/meals.db`.
- **SC-005**: An existing `data/` directory from a prior run is reused without errors or data loss.
- **SC-006**: `MEALME_DATA_DIR=data/` (with trailing slash) behaves identically to `MEALME_DATA_DIR=data`.
