# Loro FFI

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fgithub.com%2Floro-dev%2Floro-ffi%2Fraw%2Frefs%2Fheads%2Fmain%2FCargo.toml&query=%24.package.version&label=version)](Cargo.toml)
[![Rust](https://img.shields.io/badge/rust-1.0+-orange.svg)](https://www.rust-lang.org)

Foreign Function Interface (FFI) bindings for [Loro](https://loro.dev), a high-performance CRDT library for building collaborative applications.

## Overview

Loro FFI provides cross-language bindings for the Loro CRDT library, enabling developers to use Loro's powerful collaborative editing capabilities in various programming languages. Built with [UniFFI](https://mozilla.github.io/uniffi-rs/), this library generates clean, idiomatic bindings for multiple target languages.

JSONPath tooling now includes `subscribe_jsonpath`, a lightweight notification hook that fires (with possible false positives) when changes might affect a query—ideal for debounced or throttled watchers.

## Supported Languages

- [loro-swift](https://github.com/loro-dev/loro-swift): **Swift** Binding for Loro
- [loro-py](https://github.com/loro-dev/loro-py): **Python** Binding (pyo3) for Loro
- [loro-react-native](https://github.com/loro-dev/loro-react-native): **React Native** Turbo Module Binding for Loro
- [loro-cs](https://github.com/sensslen/loro-cs): **C#** Binding for Loro (community)
- [loro-go](https://github.com/aholstenson/loro-go): **Go** Binding for Loro (community)

## CHANGE LOG

You can find the updates [here](./doc/CHANGELOG.md)

## Persisting pending imported updates

`LoroDoc::export_pending_updates()` exports remote operations that were imported but
cannot yet be applied because causal dependencies are missing. These are different
from local edits in an uncommitted transaction. Ordinary snapshot and update exports
do not preserve these pending operations.

The Rust return type is `Result<Option<Vec<u8>>, LoroEncodeError>`; the UniFFI
declaration is `[Throws=LoroEncodeError] bytes? export_pending_updates()`.
Generated bindings return Swift `Data?` and Python `typing.Optional[bytes]`.
`None` (native `nil`/`None`) means there are no pending operations; it is not an
empty byte array. The method does not consume pending operations, commit local
edits, or advance the applied version vector.

Encoding failures are thrown as `LoroEncodeError`, including `InternalError`.
Swift receives `LoroEncodeError.InternalError(message:)`; Python receives
`LoroEncodeError.InternalError`. Both preserve the core's display message.
An encoding error is distinct from an absent pending result.

To preserve a checkpoint, export both a snapshot and pending updates without
allowing edits or imports between the exports, then save the pair atomically. If
pending updates are absent, clear any previously stored pending bytes. If either
export fails, keep the previous complete checkpoint. Restore by importing the
snapshot and, when present, the pending bytes through the ordinary `import`
method. The restored pending operations still wait for the actual missing
dependencies; for example, importing B without its dependency A, saving and
restoring, then importing A applies both A and B. Saving pending bytes does not
establish that a document is synced or that a sync cursor can be advanced or
history compacted.

The default dependency is pinned to the [core review commit](https://github.com/loro-dev/loro/commit/c48e4c7e7a2faa3753723b721eb3b82b09406055)
that adds this API. The checked-in lockfile uses that exact revision, so another
machine can build and review this change without a local core checkout. Replace
this review dependency with a released core version containing the API before
publishing an FFI release; coordinate that dependency update with the core change.

To validate the locked dependency and generated bindings, run:

```bash
python3 scripts/verify-pending-updates.py
```

Add `--offline` to use cached Cargo dependencies. The script makes an isolated
copy with `cdylib` output while preserving the default dependency and lockfile. It
runs the Rust tests, generates Swift and Python bindings, and runs the Python
recovery and error tests. On macOS with `swiftc`, it also runs the Swift recovery
and error tests. Set `CARGO_TARGET_DIR` to reuse a build cache. The temporary copy
and generated artifacts remain available for inspection at the printed path.

For development against a different core checkout, pass its workspace path as
an optional argument. This changes the dependency only in the temporary copy and
validates that local checkout instead of the checked-in core revision.

## Build Binding

Read the UniFFI [documentation](https://mozilla.github.io/uniffi-rs/) and modify the [uniffi.toml](./uniffi.toml) configuration in this repository as needed.

Add this repository as a git submodule for your bindings:

```bash
git submodule add https://github.com/loro-dev/loro-ffi.git loro-ffi
git add .gitmodules loro-ffi
```

Build language-specific wrapper layers around the current UniFFI binding results to provide a better developer experience, including but not limited to:

- **Callback Function Wrapping**: Provide direct closure support for functions that require callback parameters, such as `LoroDoc::subscribe()`, `UndoManager::set_on_push()`, and `EphemeralStore::subscribe()`.

- **Native Type Extensions**: Implement `LoroValueLike` `ContainerIdLike` extensions for built-in types in target languages, enabling direct parameter usage:
  ```ts
  LoroMap.insert("key", "value");
  LoroMap.insert("key", 123);
  LoroMap.insert("key", true);
  LoroMap.insert("key", [1, 2, 3]);
  // ... and more
  ```

- **Unified Export Interface**: Create a unified entry point for `LoroDoc::export`:
  ```ts
  class LoroDoc {
    function export(mode: ExportMode) {
        // Wraps:
        // exportSnapshot()
        // exportUpdates(from)
        // exportUpdatesInRange(spans)
        // exportShallowSnapshot(frontiers)
        // exportStateOnly(frontiers)
        // exportSnapshotAt(frontiers)
    }
  }
  ```

- **Unified Container Creation Interface**: Provide unified entry points for creating child containers in `LoroMap`/`LoroList`/`LoroMovableList`:
  ```ts
  class LoroMap {
    function insertContainer(key: string, child: Container) {
        // Wraps:
        // insertListContainer
        // insertMapContainer
        // insertTextContainer
        // insertTreeContainer
        // insertMovableListContainer
        // insertCounterContainer
    }
    function getOrCreateContainer() {}
    // ... and more
  }
  ```

- **Standard Language Interfaces**: Implement essential interfaces (equality, comparison, ordering, etc.) for necessary data types as required by target programming languages.

- **Additional Enhancements**: And many other improvements to enhance usability and developer experience.

## Community

- **Discord**: [Join our Discord server](https://discord.gg/tUsBSVfqzf)
- **Twitter**: [@loro_dev](https://twitter.com/loro_dev)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
