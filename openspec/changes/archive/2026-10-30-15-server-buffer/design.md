# Design — 15

## Decisions

- `buffer_size = 0` means **auto**: take the server's size. It becomes the config default (`crates/app/src/config.rs`), so a fresh install never reconfigures the owner's graph.
- `start_jack` calls `set_buffer_size` only for a non-zero size; the stats read the server's real size (`client.buffer_size()`) either way — already the case.
- cpal fallback: with auto, the stream is built from the device's default config instead of forcing the configured size.
- Settings screen: the buffer row cycles 64 → 128 → 256 → 512 → 1024 → auto → 64…; auto saves 0. The status line keeps showing the actual size.
- The owner's config keeps his explicit numbers untouched; the migration is only the default for configs without the line (his file already says 1024, which matches the graph — no reconfigure either).

## Open questions

1. Should a mismatch (server 1024, owner's explicit 256) warn that the start will reconfigure the graph? Taken: no, the status line already shows the actual size after start.
