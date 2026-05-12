# Rust GUI Efficiency Study

This workspace compares three Rust programs, with the main focus on desktop GUI efficiency:

- `Win32_API`: a low-level Windows desktop application built directly with the Win32 API.
- `modern_ops_studio`: a modern themed desktop application built with `eframe/egui`.
- `hello_rust`: a small console program used to confirm that Rust, Cargo, and compilation are working.

The goal is to evaluate Rust GUI development from the practical side: visual result, executable size, runtime feel, dependency cost, and development complexity.

## Quick Results

| Project | GUI stack | Release executable | Approx. size | Main lesson |
| --- | --- | --- | ---: | --- |
| `Win32_API` | Raw Win32 API through Rust FFI | `Win32_API/dist/RustOpsDesk.exe` | 146.5 KB | Very small and fast, but verbose and low-level |
| `modern_ops_studio` | `eframe/egui` with OpenGL backend | `modern_ops_studio/dist/ModernOpsStudio.exe` | 5.87 MB | Modern UI with much better developer speed |
| `hello_rust` | Console only | built with `cargo run` | tiny | Rust toolchain sanity check |

## What This Shows About Rust GUI Apps

Rust can produce very small native desktop programs when using system APIs directly. The Win32 version is the clearest example: the executable is around 146 KB, has no third-party GUI dependencies, and starts as a normal native Windows process. The cost is that the source code is much more manual: layout, colors, fonts, events, resizing, and control IDs all need to be handled directly.

Rust can also produce modern-looking desktop tools with a much faster development workflow. The `modern_ops_studio` app uses `eframe/egui`, so the UI is easier to design, resize, theme, and extend. The output is larger because it includes a GUI framework, rendering support, fonts, and supporting crates. Even so, a roughly 6 MB executable is still lightweight compared with many browser-based desktop stacks.

## Build Commands

From this workspace root:

```powershell
cd Win32_API
cargo build --release
```

```powershell
cd modern_ops_studio
cargo build --release
```

```powershell
cd hello_rust
cargo run
```

## File Map

```text
README.md
.gitignore
hello_rust/
  README.md
  Cargo.toml
  Cargo.lock
  src/main.rs
Win32_API/
  README.md
  Cargo.toml
  Cargo.lock
  src/main.rs
  dist/RustOpsDesk.exe
modern_ops_studio/
  README.md
  Cargo.toml
  Cargo.lock
  src/main.rs
  dist/ModernOpsStudio.exe
```

## Git Notes

The `target/` folders are ignored because Cargo can rebuild them at any time and they can become very large. The `dist/` folders are intentionally not ignored here because they contain the demonstration executables used for comparing output size.
