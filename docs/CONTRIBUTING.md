# Contributing

```bash
cargo fmt && cargo clippy --all-targets && cargo test && cargo run
```

Run from repo root so `resources/` resolves.

- Keep rules in `battle/` / `pokemon/`; keep `app/` as UI glue.
- Don't panic on missing files.
- Preserve crate-root compatibility exports.

## Sell / distribute checklist
1. Replace copyrighted music with original/licensed audio
2. No official Nintendo sprite/sound rips
3. Fan-game disclaimer + MIT license
4. `cargo build --release` smoke-test on target platforms
