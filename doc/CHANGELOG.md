# Change Log

## Unreleased

- add `LoroDoc::export_pending_updates` as a thin wrapper around the core method,
  returning optional bytes and throwing `LoroEncodeError`; absent bytes mean no
  pending imported operations, not an empty byte array; generated types are
  Swift `Data?` and Python `typing.Optional[bytes]`
- expose `LoroEncodeError::InternalError` in the UDL error mapping, preserving
  encoding error messages in Swift/Python instead of an unexpected Rust panic;
  cover the error path through public JSON import and pending export
- document saving and restoring a consistent snapshot and pending-updates pair;
  export preserves pending operations without committing local edits or advancing
  the applied version vector
- pin the default dependency to core review commit
  `c48e4c7e7a2faa3753723b721eb3b82b09406055` so the API builds on other
  machines; replace this review dependency with a released core version before
  publishing, coordinated with the core change
- add an isolated validation script that preserves the locked dependency by
  default, with an optional local-core override for development

## v1.10.3

- bump crate and dependency to loro 1.10.3
- add `LoroDoc::subscribe_jsonpath` for lightweight JSONPath change notifications

## v1.10.0

- bump crate and dependency to loro 1.10.0
- add text UTF-16 helpers, `slice_delta`, position conversion via `PosType`, and UTF-8/UTF-16 mark/unmark helpers
- expose `get_container` from `LoroDoc` and surface new `LoroError::ImportUnsupportedEncodingMode`

## v1.6.0

- bump version to loro 1.6.0

## v1.5.12

### Fix
- fix EventHint cross-container merge bug 

## v1.5.11

### New

- VersionVector
    - `try_update_last(ID id)`: Update the end counter of the given client if the end is greater. Return whether updated
    - `to_hashmap()`: convert to `Hashmap<PeerID, Counter>`

- Frontier
    - `is_empty()`: whether the frontier is empty
    - `to_vec()`: convert the frontier to `Vec<ID>`
