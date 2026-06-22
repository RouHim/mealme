# Feature Specification: Optional Meal Images

**Created**: 2026-06-14
**Status**: Approved
**Input**: lets add optional images to meals

## Goal

Allow users to attach an optional image to each meal. Uploaded images are accepted in any common format, automatically converted to 82%-quality JPEG server-side, and stored inline in the SQLite database for single-file portability. The meal list renders CSS-constrained thumbnails for meals that have an image; clicking a thumbnail opens a lightbox overlay showing the full image.

## User Scenarios

### Scenario 1 - Add image when creating a meal (P1)
A user filling out the "New Meal" form selects an image file via a file input. After submitting, the meal appears in the list with a thumbnail. The image was converted to JPEG on the server during upload.

**Acceptance**
1. Given the create-meal form is open, when the user selects a valid image file and submits the form, then the meal is created and its image is stored as a JPEG.
2. Given the create-meal form is open, when the user selects a file that is not a valid image and submits, then the server rejects the upload with a clear error message and the meal is not created.

### Scenario 2 - Add, replace, or remove image on an existing meal (P1)
A user edits a meal. The edit form shows whether an image currently exists. The user can upload a new image (replacing any existing one) or remove the current image without deleting the meal.

**Acceptance**
1. Given a meal has no image, when the user edits it and uploads an image, then the meal shows a thumbnail in the list.
2. Given a meal has an image, when the user edits it and uploads a different image, then the new image replaces the old one.
3. Given a meal has an image, when the user edits it and chooses to remove the image, then the meal no longer has an image and no broken placeholder is shown.

### Scenario 3 - View thumbnail in meal list (P1)
The meal list page renders a small thumbnail for each meal that has an image. Meals without images show no placeholder or broken-image indicator.

**Acceptance**
1. Given a meal has an image, when the meal list is displayed, then a CSS-constrained thumbnail of that image is visible in the meal's row.
2. Given a meal has no image, when the meal list is displayed, then no `<img>` element or placeholder is rendered for that meal.

### Scenario 4 - View full-size image in a lightbox (P1)
Clicking a meal's thumbnail opens the full image in a lightbox overlay. Clicking outside the image or pressing Escape closes the overlay.

**Acceptance**
1. Given a thumbnail is visible in the meal list, when the user clicks it, then a lightbox overlay appears showing the full-size image.
2. Given the lightbox is open, when the user clicks outside the image or presses Escape, then the lightbox closes and the meal list is visible again.

## Functional Requirements

- **FR-001**: Each meal may have zero or one image. Multiple images per meal are not supported.
- **FR-002**: The server MUST accept JPEG, PNG, WebP, GIF, and BMP input formats. On upload, the server MUST convert the image to JPEG at 82% quality before storage.
- **FR-003**: The image MUST be stored as a BLOB column in the `meals` table, alongside the meal's other fields. No separate images table or filesystem directory.
- **FR-004**: A dedicated endpoint `GET /api/meals/:id/image` MUST return the JPEG binary with `Content-Type: image/jpeg`. When the meal has no image, the endpoint MUST return `204 No Content`.
- **FR-005**: The meal list response (`GET /api/meals` and `GET /api/meals/:id`) MUST include a `has_image: boolean` field so the frontend knows whether to render an `<img>` tag without fetching the binary.
- **FR-006**: The meal create and update endpoints MUST accept an optional image upload. For update, an explicit "remove image" signal MUST be distinguishable from "no image change."
- **FR-007**: The frontend meal list MUST render a CSS-constrained thumbnail (`<img>` with constrained dimensions) for each meal where `has_image` is true, sourcing from the image endpoint.
- **FR-008**: Clicking a thumbnail MUST open a lightbox overlay (CSS-driven, no external library) showing the same image at its natural size. The overlay MUST be closable via click-outside and Escape key.
- **FR-009**: Meals with `has_image: false` MUST NOT render any `<img>` element, broken-image fallback, or placeholder icon in the list.
- **FR-010**: Non-image file uploads MUST be rejected by the server with a `400 Bad Request` and a JSON error body indicating the file was not a recognizable image format.
- **FR-011**: Corrupt or un-decodable image data in an otherwise valid file MUST be rejected with a `400 Bad Request` and a descriptive error.

## Key Entities

- **Meal**: Gains two new columns:
  - `image` — nullable BLOB containing the 82%-quality JPEG, or NULL when no image is attached.
  - `image_content_type` — TEXT, always `"image/jpeg"` when `image` is non-NULL, NULL otherwise. Stored to simplify future format flexibility.

## Edge Cases

- **Very large originals**: No file-size limit is enforced. The server converts to 82% JPEG regardless of input size. At the projected scale (~1000 meals, ~1–3 MB compressed per image), total storage remains well within SQLite's practical limits.
- **Concurrent uploads**: Not a concern — the app is single-user and SQLite serializes writes.
- **Image removal on meal delete**: The image is stored in the same `meals` row; deleting the meal deletes the image automatically.
- **Existing meals**: Pre-feature meals have `image = NULL` and `has_image = false`. They continue to work with no changes required.
- **Large DB after many uploads**: SQLite handles databases up to ~281 TB. At 1000 meals × 3 MB, the DB is ~3 GB, which is routine. The `VACUUM` command can reclaim space from deleted images if needed.
- **Upload with same filename as prior image**: Filenames are irrelevant — the server processes the bytes and discards the original filename.

## Research Notes

- SQLite reads small BLOBs (~10 KB) ~35% faster than individual files on disk and uses ~20% less space due to tighter packing. ([sqlite.org/fasterthanfs.html](https://sqlite.org/fasterthanfs.html))
- The Fossil SCM stores all version-controlled files (binary and text) as BLOBs in SQLite — production-proven at scale for 15+ years. ([sqlite.org forum](https://sqlite.org/forum/forumpost/e18c43fc797fcdb7))
- Microsoft Research found database BLOB storage faster than filesystem for objects under 250 KB–1 MB. Most 82%-quality JPEGs fall within this window. ([Microsoft Research paper](https://www.microsoft.com/en-us/research/publication/to-blob-or-not-to-blob-large-object-storage-in-a-database-or-a-filesystem/))
- JPEG at 82% quality produces files roughly 0.5–3 MB for typical photos, depending on resolution and content complexity. ([toolstud.io](https://toolstud.io/photo/filesize.php), dpreview forums)

## Assumptions

- Server-side JPEG conversion uses a pure-Rust image processing crate (e.g., the `image` crate) with no external system dependencies.
- No client-side image resizing or format conversion occurs before upload; the server handles all processing.
- The lightbox overlay is implemented with plain CSS and Svelte conditional rendering — no JavaScript animation library or modal framework.
- Thumbnails are the same JPEG file served by the image endpoint, visually constrained via CSS `max-width` / `max-height` — no separate thumbnail asset is generated or stored.
- No `Cache-Control`, `ETag`, or `If-Modified-Since` headers are needed for the image endpoint, as the app serves only localhost.
- Multipart form data is used for image upload on create and update (not base64-in-JSON), keeping the JSON body clean and avoiding payload inflation.
- The "remove image" signal on update is an explicit field or sentinel value (e.g., `image_action: "remove"` or a zero-length multipart part), not inferred from absence.

## Success Criteria

- **SC-001**: Creating a meal with a valid image file results in a meal that displays a thumbnail in the list and serves its image via the image endpoint.
- **SC-002**: Clicking a meal's thumbnail opens a lightbox overlay showing the full image, and the overlay closes on click-outside or Escape.
- **SC-003**: Uploading an image to a meal that previously had no image adds the image and sets `has_image` to `true`.
- **SC-004**: Uploading a new image to a meal that already had an image replaces the old image with the new one.
- **SC-005**: Removing an image from a meal sets `has_image` to `false` and the image endpoint returns `204 No Content`, without deleting the meal.
- **SC-006**: Uploading a non-image file (e.g., a PDF or text file) as a meal image returns `400 Bad Request` with a clear error message.
- **SC-007**: The meal list renders no broken images, placeholders, or alt-text fallbacks for meals without images.
- **SC-008**: All existing meals created before this feature continue to display and function normally, with `has_image: false`.
- **SC-009**: Accepting a JPEG, PNG, WebP, GIF, or BMP image all result in a valid JPEG stored in the database and served correctly.
