# Win32 API Rust Desktop App

This project is a native Windows desktop application written in Rust using direct Win32 API calls through FFI. The Cargo package name is `rust_desktop_gui`, while the folder is named `Win32_API`.

## Purpose

This app demonstrates the smallest and most direct path to a Rust desktop GUI on Windows:

- no external GUI framework
- no Rust GUI crate
- direct calls to `user32`, `gdi32`, and `kernel32`
- manual window creation, controls, colors, fonts, events, and resizing

It was built to show how efficient Rust can be when it talks directly to the operating system.

## Result

The release executable is:

```text
dist/RustOpsDesk.exe
```

Approximate size:

```text
146.5 KB
```

This is very small because the app relies on Windows system libraries instead of bundling a full GUI toolkit.

## What The App Contains

- crowded internal operations dashboard
- module navigation
- ticket intake form
- operational action buttons
- inventory watch list
- work queue
- event stream
- custom Win32 colors and fonts
- manual resize behavior through `WM_SIZE`
- minimum window sizing through `WM_GETMINMAXINFO`

## Efficiency Notes

### Strengths

- extremely small executable
- native Windows controls
- very low dependency surface
- fast startup in normal use
- full control over the Win32 message loop

### Tradeoffs

- code is verbose
- UI styling is manual
- layout and resizing must be implemented by hand
- portability is Windows-only
- development speed is slower than using a GUI framework

## Build

```powershell
cargo build --release
```

Cargo writes the optimized build to:

```text
target/release/rust_desktop_gui.exe
```

The prepared demonstration copy is:

```text
dist/RustOpsDesk.exe
```

## Important Files

```text
Cargo.toml      Package metadata. No third-party dependencies.
Cargo.lock      Locked Rust package graph.
src/main.rs     Full Win32 API application source.
dist/           Prepared release executable for comparison.
target/         Cargo build output. Safe to delete and ignored by Git.
```
