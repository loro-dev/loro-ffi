# Unreadable lazy history fixture

`unparsable-history.snapshot.bin` is the synthetic public Loro regression fixture
from [core PR #1177](https://github.com/loro-dev/loro/pull/1177), copied unchanged
from commit `4f60d921d820de3522624a436de77f994a6622b8`, path
`crates/loro-wasm/tests/fixtures/unparsable-history.snapshot.bin`.

The fixture has a truncated non-frontier change block with repaired checksums.
Initial snapshot import succeeds; reads that need the block must return a typed
history/decode error. It contains no user data.

SHA-256: `016f4c06d32c92ba0f88e8d64897a8182c0c70c0f26d92ba0d12cdd3c8cb0480`.
