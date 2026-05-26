# Shared Specs

Versioned contract files that keep `vision-core`, `rules-core`, and the desktop UI aligned.

- `tile_vocab.v1.json`: canonical tile IDs used across detection, correction, and scoring.
- `recognition_result.v1.json`: recognition result shape returned by desktop commands.
- `scoring-result.v1.json`: analysis payload from rules engine.

When model or rules internals change, preserve these interfaces (or bump version explicitly).
