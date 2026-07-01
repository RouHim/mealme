# Feature Specification: LLM Import Meal Image Extraction

**Created**: 2026-07-01
**Status**: Approved
**Input**: For LLM URL/description import, try also to parse via LLM a meal image

## Goal

When importing a meal via the LLM tab (from a recipe URL or text description), the LLM should also attempt to return a meal image URL. The server downloads and converts that image to JPEG and passes it back in the `ImportDraft.image_base64` field, so the user sees a pre-filled image thumbnail in the meal form — the same UX as the existing non-LLM URL import path. This is strictly best-effort: a missing, invalid, or undownloadable image URL never blocks the recipe import.

## User Scenarios

### Scenario 1 - Import from recipe URL with meal image (P1)

A user pastes a recipe URL into the LLM tab hint field. The server fetches the page, extracts readable text, and sends it to the LLM. The LLM returns the recipe (name, ingredients, instructions) plus an `imageUrl` it identifies from the recipe's web context. The server downloads the image, converts it to JPEG, and the import draft includes the image. The meal form opens with a pre-filled image thumbnail.

**Acceptance**

1. Given the LLM returns a valid `imageUrl` pointing to an accessible image, When the server processes the tool result, Then the returned `ImportDraft.image_base64` is non-null and the meal form shows the image thumbnail.
2. Given the LLM returns a recipe without an `imageUrl` field, When the server processes the tool result, Then the returned `ImportDraft.image_base64` is null and the import completes with recipe data only (no error).
3. Given the LLM returns an `imageUrl` that is unreachable or returns non-image content, When the server attempts the download, Then the download fails silently and `ImportDraft.image_base64` is null (no error surfaced to the user).

### Scenario 2 - Import from text description with best-effort image (P1)

A user types a text description of a dish (no URL) into the LLM tab hint field. The LLM returns the recipe and may include an `imageUrl` from its own knowledge for the described dish. If the URL is valid and downloadable, the image is included; if not, the recipe imports without an image.

**Acceptance**

1. Given the LLM returns an `imageUrl` for a description-only import and the URL is downloadable, When the server processes the tool result, Then `ImportDraft.image_base64` is non-null and the form shows the thumbnail.
2. Given the LLM does not return an `imageUrl` for a description-only import, When the server processes the tool result, Then the recipe imports normally with `image_base64` null (no error, no indication an image was expected).

### Scenario 3 - User-uploaded photo takes precedence (P2)

A user uploads a photo file alongside a URL or description hint. The LLM returns both the recipe and an `imageUrl`. The user's uploaded file is used for the form image; the LLM-returned `imageUrl` is not downloaded.

**Acceptance**

1. Given a user uploads an image file and the LLM also returns an `imageUrl`, When the server processes the import, Then the uploaded file populates the form image and the `imageUrl` is not fetched (download is skipped when a user-uploaded image is present).

## Functional Requirements

- **FR-001**: The `extract_recipe` LLM tool schema (`src/llm_import.rs`, `recipe_tool()` function) gains an optional `imageUrl` field: `{ "type": "string", "description": "A URL of a photo of the finished dish, if one can be identified from the recipe context" }`, not added to the `required` array.

- **FR-002**: The system prompt (`SYSTEM_PROMPT` constant in `src/llm_import.rs`) instructs the LLM to provide an `imageUrl` when it can identify one from the recipe's context (URL page text or its own knowledge for description-only imports), and to omit the field when it cannot.

- **FR-003**: The `LlmRecipeDraft` deserialization struct (`src/llm_import.rs`) gains an optional `image_url: Option<String>` field (with `#[serde(default)]` and `#[serde(rename = "imageUrl")]` to match the tool schema's camelCase key).

- **FR-004**: The `build_draft_from_tool_args` function (`src/llm_import.rs`) reads `draft.image_url`. If present and non-empty (after trim), it attempts to download and convert the image by calling `recipe::try_download_image` with a `reqwest::Client` (30s timeout, matching the existing pattern in `recipe::fetch_and_parse`). On success, the JPEG bytes are base64-encoded and set as `ImportDraft.image_base64`. On any failure (network error, non-success status, decode error), `image_base64` remains `None` — no error is returned.

- **FR-005**: When the user uploads an image file (the multipart `image` field is present and non-empty in `import_from_llm`), the LLM's `imageUrl` is not downloaded. The uploaded image flows through its existing path. The imageUrl download only occurs when the LLM returns an imageUrl AND no user-uploaded image was provided. This precedence is enforced because `build_draft_from_tool_args` runs after the LLM call (which has no knowledge of uploads) — to implement this, `import_via_llm` or `build_draft_from_tool_args` must receive a flag indicating whether a user-uploaded image was provided; if `true`, the imageUrl download is skipped.

- **FR-006**: Downloaded images go through the same `convert_to_jpeg` pipeline (`src/image.rs`) as all other meal images: downscale to 3840px on the longer edge, JPEG quality 82, MIME type `image/jpeg`.

- **FR-007**: No new API endpoint, no new frontend type, no new i18n key, and no new Cargo dependency is introduced. The existing `ImportDraft.image_base64` field, the frontend's base64-to-File handling (`web/src/routes/meals/+page.svelte` lines 74–78), and the existing `recipe::try_download_image` function are reused.

## Key Entities

- **`LlmRecipeDraft`** (`src/llm_import.rs`): Private deserialization struct for the LLM tool's JSON output. Gains the `image_url` field.
- **`ImportDraft`** (`src/recipe.rs`): The meal-shaped draft returned to the frontend. Its existing `image_base64: Option<String>` field carries the downloaded image; no structural change to this type.
- **`try_download_image`** (`src/recipe.rs`): Existing best-effort image downloader + JPEG converter. Reused as-is. Currently `async fn try_download_image(client: &reqwest::Client, url: &str) -> Option<Vec<u8>>` (private to `recipe` module — must be made `pub(crate)` or a `pub(crate)` wrapper must be added if it is not already accessible from `llm_import`).

## Edge Cases

- **LLM hallucinates an invalid/unreachable URL**: download fails → `image_base64` stays `None`, no error surfaced. This is the expected and most common failure mode.
- **LLM returns a URL pointing to non-image content** (e.g., an HTML page): `convert_to_jpeg` fails → `image_base64` stays `None`, no error.
- **LLM returns a relative URL or a URL without a scheme**: download fails (reqwest requires absolute URLs) → `image_base64` stays `None`. No URL normalization or relative-to-absolute resolution is performed.
- **LLM returns a valid image URL that is very large**: the download client has a 30-second timeout (matching `try_download_image`'s existing client config). `convert_to_jpeg` downscales to 3840px max on the longer edge. There is no byte-size cap on the downloaded image beyond the 30s timeout — consistent with the existing non-LLM URL import path (`fetch_and_parse`).
- **Both user-uploaded image file and LLM imageUrl present**: the user's uploaded file wins; the imageUrl download is skipped entirely (FR-005). The LLM never sees the uploaded image's bytes for the purpose of imageUrl selection — it already received the uploaded image as a vision input if provided.
- **LLM returns `imageUrl` as an empty string or whitespace-only**: trimmed to empty → treated as absent, no download attempted.

## Assumptions

- The LLM is instructed via an edit to `SYSTEM_PROMPT` to provide `imageUrl` when it can identify one from recipe context. For description-only imports, the LLM may return a known image URL or omit the field. No guarantee is made that a URL will be present or valid — this is inherent to the chosen approach (LLM identifies URLs from context, not server-side JSON-LD extraction).
- Image URL download reuses the existing 30s-timeout `reqwest::Client` pattern from `recipe::try_download_image` (a fresh client built per download, identical to `recipe::fetch_and_parse`). No new client configuration or retry logic is introduced.
- `recipe::try_download_image` is currently a private function in the `recipe` module. To call it from `llm_import.rs`, it must be made `pub(crate)`, or a `pub(crate)` wrapper added. This visibility change is the only structural change to `recipe.rs` and does not alter the function's behavior or signature.
- The user-approved precedence rule (upload wins over LLM imageUrl) requires passing a boolean flag (`has_user_image: bool`) from `import_from_llm` in `routes.rs` into `import_via_llm` / `build_draft_from_tool_args` in `llm_import.rs`, so the download is skipped when the user uploaded a file. This is a signature change to `import_via_llm` and `build_draft_from_tool_args`, and every call site (currently one: `routes.rs:441`) must be updated — a clean cutover with no default-bool compatibility shim.
- No new i18n keys are needed because there is no new user-visible text: the image simply appears (or doesn't) in the form thumbnail, exactly as the non-LLM URL import already behaves.
- The feature applies only to the LLM import path (`/api/import/llm`). The existing non-LLM URL import (`/api/import/url`, which already downloads images via `fetch_and_parse`) and the paste import (`/api/import/paste`, which has no image) are unchanged.

## Success Criteria

- **SC-001**: Given a mocked LLM tool response containing a valid `imageUrl` pointing to an accessible JPEG, When `build_draft_from_tool_args` processes it (no user upload flag), Then the returned `ImportDraft.image_base64` is a non-null base64 string that decodes to valid JPEG bytes (verifiable in a unit test that mocks the download or uses a local `file://`/embedded image URL).

- **SC-002**: Given a mocked LLM tool response without an `imageUrl` field, When `build_draft_from_tool_args` processes it, Then the returned `ImportDraft.image_base64` is `None` and no download is attempted (verifiable: the existing `given_valid_args_when_build_draft_then_returns_draft` test still passes with `image_base64` None).

- **SC-003**: Given a mocked LLM tool response containing an `imageUrl` that is unreachable or returns non-image content, When `build_draft_from_tool_args` processes it, Then `image_base64` is `None` and no `AppError` is returned — the function returns the recipe draft successfully (verifiable with a test using an invalid URL like `http://127.0.0.1:1/nope.jpg`).

- **SC-004**: Given the `has_user_image` flag is `true` and the LLM returns a valid `imageUrl`, When `build_draft_from_tool_args` processes it, Then `image_base64` is `None` and no download is attempted (verifiable: assert no network call is made — the download is skipped before any HTTP request).

- **SC-005**: `cargo test` passes with the existing suite plus new tests for the image-url download path (success, absent, failure, upload-precedence), and `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt` are clean.

- **SC-006**: `cd web && npm run check` passes (no frontend changes are required, confirming no type breakage from the backend change).
