# Change Log

## v1.5.11

### New

- VersionVector
    - `try_update_last(ID id)`: Update the end counter of the given client if the end is greater. Return whether updated
    - `to_hashmap()`: convert to `Hashmap<PeerID, Counter>`

- Frontier
    - `is_empty()`: whether the frontier is empty
    - `to_vec()`: convert the frontier to `Vec<ID>`