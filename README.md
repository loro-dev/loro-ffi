# Loro FFI

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-1.5.8-green.svg)](Cargo.toml)
[![Rust](https://img.shields.io/badge/rust-1.0+-orange.svg)](https://www.rust-lang.org)

Foreign Function Interface (FFI) bindings for [Loro](https://loro.dev), a high-performance CRDT library for building collaborative applications.

## Overview

Loro FFI provides cross-language bindings for the Loro CRDT library, enabling developers to use Loro's powerful collaborative editing capabilities in various programming languages. Built with [UniFFI](https://mozilla.github.io/uniffi-rs/), this library generates clean, idiomatic bindings for multiple target languages.

## Supported Languages

- [loro-swift](https://github.com/loro-dev/loro-swift): **Swift** Binding for Loro
- [loro-py](https://github.com/loro-dev/loro-py): **Python** Binding (pyo3) for Loro
- [loro-react-native](https://github.com/loro-dev/loro-react-native): **React Native** Turbo Module Binding for Loro
- [loro-cs](https://github.com/sensslen/loro-cs): **C#** Binding for Loro (community)
- [loro-go](https://github.com/aholstenson/loro-go): **Go** Binding for Loro (community)

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
  ```
  LoroMap.insert("key", "value");
  LoroMap.insert("key", 123);
  LoroMap.insert("key", true);
  LoroMap.insert("key", [1, 2, 3]);
  // ... and more
  ```

- **Unified Export Interface**: Create a unified entry point for `LoroDoc::export`:
  ```
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
  ```
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
