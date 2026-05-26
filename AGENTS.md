# AGENTS.md

Guidance for coding agents working in this repository.

## Project Overview

This repo contains desktop tools for Japanese mahjong workflows.

- `sparrow-vision/`: Tauri 2 + React app for photo-based riichi mahjong recognition and scoring.
- `ml/sparrow-mark/`: Tauri 2 + React app for ML labeling/marking workflows.
- `ml/model/last.onnx`: ONNX recognition model used by `sparrow-vision`.
- `shared/specs/`: versioned JSON contracts shared by recognition, rules, and UI code.

Keep frontend and backend structure clean. Prefer small feature modules over large catch-all files.

## Common Commands

Run from the repository root unless noted.

- `cargo test`: run Rust workspace tests.
- `pnpm --filter sparrow-vision build`: type-check and build the `sparrow-vision` frontend.
- `pnpm vision:dev`: run the `sparrow-vision` Tauri app in development.
- `pnpm vision:build`: build the `sparrow-vision` Tauri app.
- `pnpm mark:dev`: run the marker app in development.
- `pnpm mark:build`: build the marker app.

For frontend-only checks inside `sparrow-vision/`:

- `.\node_modules\.bin\tsc.CMD --noEmit`
- `.\node_modules\.bin\vite.CMD build`

## Contracts And Data Shapes

Treat `shared/specs/*.json` as the source of truth for cross-boundary payloads.

- `tile_vocab.v1.json`: canonical tile IDs.
- `recognition_result.v1.json`: recognition command output.
- `scoring-result.v1.json`: scoring command output.

If a payload shape changes, update the spec intentionally or keep the implementation compatible. Do not silently drift from these schemas.

## sparrow-vision Backend Notes

Backend code lives in `sparrow-vision/src-tauri/src/`.

- `commands.rs`: thin Tauri command layer.
- `types.rs`: serializable DTOs aligned with `shared/specs`.
- `vision/`: image preprocessing, ONNX inference, YOLO output parsing, result assembly, and hand grouping.
- `scoring/`: tile mapping, validation, dora conversion, and `riichi-calc` adapter.

Recognition model details:

- Use `ml/model/last.onnx`.
- Tauri bundles it as a resource at `model/last.onnx`.
- Development may fall back to the repo path.
- The model is large; keep model/session loading cached and avoid reloading per upload.
- The current YOLO26 ONNX output is post-processed detection rows with shape `[1, N, 6]`.
- Detection rows are `x1, y1, x2, y2, confidence, class_id` in model letterbox coordinates.
- Backend maps model `x1/y1/x2/y2` back to original image pixels and exposes `bbox` as `x, y, width, height`.
- The YOLO26 model already applies NMS. Do not add backend NMS unless the model export changes.
- Recognition debug fixture images live in `sparrow-vision/src-tauri/tests/fixtures/recognition/images/`.
- Ignored debug tests in `vision/recognizer.rs` can print model output, print assembled `RecognitionResult`, or save an annotated image.
- Debug annotated images must be written to the system temp directory, not into fixture directories.

Scoring details:

- `riichi-calc` is a pinned git dependency.
- Do not rely on private fields from `calc_hand()` output.
- Build scoring results from public APIs such as `parse_hand`, `Finder::find_hand`, and `calculator::score::calc_score`.
- UI dora input is dora indicators; backend converts to actual dora before scoring.
- `f1` through `f8` may be displayed but must not be accepted for riichi scoring.

## sparrow-vision Frontend Notes

Frontend code lives in `sparrow-vision/src/`.

- `features/upload`: file selection and drag/drop.
- `features/recognition`: image preview and detection overlay.
- `features/hand-editor`: tile, meld, pending-tile editing.
- `features/scoring`: scoring settings and result display.
- `shared`: API client, tile constants, shared types, reusable UI.

Use CSS tile badges for v1. Keep controls compact and tool-like; this app is an operational workbench, not a marketing page.

When rendering detection boxes, remember backend bbox values are in original image pixels. Scale them to displayed image dimensions.

## Editing Guidelines

- Preserve existing user changes. Do not revert unrelated work.
- Prefer `rg` / `rg --files` for search.
- Use `apply_patch` for manual file edits.
- Keep comments sparse and useful.
- Keep DTO names and serialized field names stable unless the shared spec is updated.
- Avoid unrelated refactors while fixing a focused issue.

## Verification Expectations

For changes touching Rust backend behavior:

- Run `cargo test`.

For changes touching `sparrow-vision` frontend:

- Run `.\node_modules\.bin\tsc.CMD --noEmit` from `sparrow-vision/`.
- Run `.\node_modules\.bin\vite.CMD build` from `sparrow-vision/`.

For Tauri packaging/resource changes:

- Ensure `tauri.conf.json` resources still point to existing files.
- Prefer `pnpm vision:build` when feasible.

## Known Constraints

- `ml/model/last.onnx` is ignored as a model artifact but is required locally for recognition.
- Recognition is a best-effort starting point. User correction is part of the intended workflow.
- v1 scoring is for completed winning hands only; do not add listening/wait analysis unless explicitly requested.
