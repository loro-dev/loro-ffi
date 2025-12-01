# Change Log

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
