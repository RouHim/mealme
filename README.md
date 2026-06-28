<p align="center">
  <img src=".github/readme/banner.svg" width="600">
</p>

<p align="center">
  <a href="https://github.com/RouHim/mealme/actions/workflows/ci.yml"><img src="https://github.com/RouHim/mealme/actions/workflows/ci.yml/badge.svg" alt="CI/CD"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/RouHim/mealme/releases/latest"><img src="https://img.shields.io/badge/arch-x86__64%20%7C%20arm64-blue" alt="Architecture: x86_64 | arm64"></a>
  <img src="https://img.shields.io/badge/renovate-enabled-brightgreen.svg" alt="Renovate enabled">
</p>

## Features

- **Local-first meal management** — add, edit, search, and delete meals with normalized ingredient rows and quantities; SQLite-backed, runs entirely from a single binary.
- **Weekly meal planner** — `/planner` route exposes a week calendar for generating and managing weekly plans; plans aggregate ingredients and sum numeric quantities.
- **Recipe import** — paste raw HTML/JSON-LD or fetch from a URL; the existing scraper normalizes into the same review-and-save `ImportDraft` workflow.
- **LLM-powered recipe parsing** — attach a photo or text hint and a vision-capable LLM (OpenAI, Anthropic, Gemini, Groq, Ollama, and many more via `genai`) returns a structured recipe draft via a constrained tool definition.
- **Single static binary** — entire Svelte 5 SPA embedded via `rust-embed`; no runtime Node, no nginx, no system SQLite (bundled via `sqlx`'s `sqlite` feature).
- **Dual theme** — light/dark with `prefers-color-scheme`, ambient photo background, manual theme toggle with `localStorage` persistence.

## Quick start

### From source

```bash
git clone https://github.com/RouHim/mealme.git
cd mealme
cargo run --release
```

Open <http://127.0.0.1:11341>.

### Pre-built binary

```bash
# x86_64 / amd64
curl -L -o mealme https://github.com/RouHim/mealme/releases/latest/download/mealme-x86_64-unknown-linux-musl
chmod +x mealme
./mealme

# arm64 / aarch64
curl -L -o mealme https://github.com/RouHim/mealme/releases/latest/download/mealme-aarch64-unknown-linux-musl
chmod +x mealme
./mealme
```

The server listens on `127.0.0.1:11341` and persists data to `./data/meals.db` (override the directory with the `MEALME_DATA_DIR` env var).

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
| POST | `/api/import/url` | Import a recipe from URL `{"url":"https://..."}` |
| POST | `/api/import/paste` | Import a recipe from raw HTML/JSON-LD `{"content":"..."}` |
| POST | `/api/import/llm` | Import a recipe via vision LLM (multipart: `model`, `hint?`, `image?`) |
| GET | `/api/plans?year=&week=` | Get a specific plan with meals and ingredient summary |
| GET | `/api/plans?year=` | List all plans for a year |
| PUT | `/api/plans/:year/:week` | Update plan meals `{"meal_ids":[1,2,3]}` |
| DELETE | `/api/plans/:year/:week` | Delete a plan |

**Ingredients** are now stored as normalized rows: each meal has one or more ingredients with optional quantity text (e.g. `"200g"`, `"2 cups"`, `"a pinch"`). Plans aggregate ingredients — identical ingredients are merged and numeric quantities summed. The `/planner` route provides a week calendar for generating and managing weekly meal plans.

Validation: meal name (1–200 chars), ingredient name (1–100 chars per line), ingredient quantity (0–50 chars), max 100 ingredient lines. All API paths are under `/api`; everything else serves the SPA frontend.

## LLM Recipe Import

The app can parse recipes from photos or text descriptions using a vision-capable LLM (e.g. GPT-4o).

### Setup

Set an API key for your provider as an environment variable:

| Provider | Env var | Example model |
|----------|---------|---------------|
| OpenAI | `OPENAI_API_KEY` | `gpt-4o-mini` |
| Anthropic | `ANTHROPIC_API_KEY` | `claude-sonnet-4-20250514` |
| Google | `GOOGLE_API_KEY` | `gemini-2.5-flash` |
| Groq | `GROQ_API_KEY` | `llama-4-maverick-17b-128e-instruct` |
| Ollama (local) | _(none)_ | `llama3.2-vision` |

```bash
OPENAI_API_KEY=sk-... cargo run --release
```

Then open the app, switch to the **Meals** page, and use the **"From photo / text"** tab:

1. Enter a model name (e.g. `gpt-4o-mini`)
2. Optionally describe the dish in the hint field
3. Optionally attach a photo of the dish or recipe
4. Click **"Parse with AI"** — review the extracted recipe, then save

At least one of hint or image is required. The LLM returns a structured recipe draft (name, ingredients, instructions) that flows into the same review-and-save workflow used by URL and paste imports.

### How it works

The backend sends a single chat completion request with a tool definition (`extract_recipe`) that constrains the LLM to return structured JSON. The response is mapped to the standard `ImportDraft` shape the frontend already understands. Requests time out after 60 seconds.

## Development

```bash
# Run all Rust tests
cargo test

# Run all frontend tests
cd web && npm test

# Build release binary
cargo build --release

# Run with debug logging
RUST_LOG=debug cargo run

# E2E tests (Playwright)
just e2e
```

## Contributing

PRs welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## License

MIT — see [LICENSE](LICENSE).
