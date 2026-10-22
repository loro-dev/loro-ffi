#!/usr/bin/env python3
"""Verify pending-update bindings in an isolated copy of the locked dependency."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("core", nargs="?", type=Path, help="Optional local Loro workspace override")
    parser.add_argument("--offline", action="store_true", help="Use cached Cargo dependencies")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    core = args.core.resolve() / "crates" / "loro" if args.core else None
    if core and not (core / "Cargo.toml").is_file():
        parser.error("core must point to a Loro workspace with crates/loro/Cargo.toml")

    # Keep artifacts for inspection; never write a path dependency into the checkout.
    work = Path(tempfile.mkdtemp(prefix="loro-ffi-pending-"))
    print(f"Validation copy and generated bindings: {work}", flush=True)
    for name in ("src", "tests"):
        shutil.copytree(repo / name, work / name)
    for name in ("Cargo.toml", "Cargo.lock", "build.rs", "uniffi.toml"):
        shutil.copy2(repo / name, work / name)

    manifest = (work / "Cargo.toml").read_text()
    if core:
        # Local development only: keep the feature list and replace the source.
        manifest, count = re.subn(
            r'(?m)^loro\s*=\s*\{.*?(,\s*features\s*=\s*\[[^\]]*\]\s*\})$',
            lambda match: "loro = { path = " + json.dumps(str(core)) + match[1],
            manifest,
        )
        if count != 1:
            raise RuntimeError("Expected exactly one loro dependency with features")
    manifest += '\n[lib]\ncrate-type = ["rlib", "cdylib"]\n'
    (work / "Cargo.toml").write_text(manifest)
    # UDL-mode Python generation otherwise defaults to a library named uniffi.
    with (work / "uniffi.toml").open("a") as config:
        config.write('\n[bindings.python]\ncdylib_name = "loro_ffi"\n')

    env = os.environ.copy()
    target = Path(env.get("CARGO_TARGET_DIR", work / "target")).resolve()
    env["CARGO_TARGET_DIR"] = str(target)
    error_fixture = work / "lowered-errors.json"
    env["LORO_FFI_ERROR_FIXTURE"] = str(error_fixture)
    env["LORO_FFI_TEST_FIXTURES"] = str(work / "tests" / "fixtures")

    def run(*command, extra_env=None):
        print("+", " ".join(map(str, command)), flush=True)
        subprocess.run(command, cwd=work, env=env | (extra_env or {}), check=True)

    def cargo(command, *arguments):
        run("cargo", command, *(["--offline"] if args.offline else []),
            *([] if core else ["--locked"]), *arguments)

    cargo("test", "--all-targets", "--features", "cli")
    if not error_fixture.is_file():
        raise RuntimeError("Rust tests did not generate the error-lowering fixture")
    cargo("test", "--doc", "--features", "cli")
    cargo("build", "--features", "cli")
    bindings = work / "bindings"
    cargo("run", "--features", "cli", "--bin", "uniffi-bindgen", "--", "generate",
          "src/loro.udl", "--language", "swift", "--language", "python",
          "--out-dir", str(bindings), "--no-format")
    library_name = "libloro_ffi.dylib" if sys.platform == "darwin" else "libloro_ffi.so"
    shutil.copy2(target / "debug" / library_name, bindings / library_name)
    run(sys.executable, "tests/bindings/test_pending_updates.py",
        extra_env={"PYTHONPATH": str(bindings)})

    if sys.platform == "darwin" and shutil.which("swiftc"):
        executable = work / "pending-updates-swift"
        run("swiftc", "-parse-as-library", "-module-cache-path", str(work / "swift-module-cache"),
            "-I", str(bindings), "-Xcc", f"-fmodule-map-file={bindings / 'loroFFI.modulemap'}",
            "-L", str(bindings), "-lloro_ffi", "-Xlinker", "-rpath",
            "-Xlinker", str(bindings), str(bindings / "loro.swift"),
            "tests/bindings/pending_updates.swift", "-o", str(executable))
        run(str(executable))
    else:
        print("Swift runtime check skipped (requires macOS and swiftc); bindings generated.")
    print(f"Passed. Inspect generated optional-byte bindings in {bindings}")


if __name__ == "__main__":
    main()
