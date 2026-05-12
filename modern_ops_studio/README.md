# Modern Ops Studio

This project is a modern themed Rust desktop application built with `eframe/egui`.

## Purpose

This app demonstrates the productive GUI path in Rust: use a Rust-native immediate-mode GUI framework to build a polished desktop tool with much less low-level code than raw Win32.

The goal is to compare the developer experience and output against the direct Win32 version.

## Result

The release executable is:

```text
dist/ModernOpsStudio.exe
```

Approximate size:

```text
5.87 MB
```

The executable is larger than the Win32 version because it includes the GUI framework, rendering backend, default fonts, window integration, and supporting crates.

## What The App Contains

- modern dark interface
- responsive layout
- side navigation
- dashboard metric cards
- ticket board
- asset health cards
- automation rules
- settings controls
- progress indicators
- custom trend chart
- event stream

## GUI Stack

The app uses:

```toml
eframe = { version = "0.34.2", default-features = false, features = ["default_fonts", "glow"] }
```

This keeps the dependency set smaller than the default `wgpu` backend while still giving a modern, hardware-accelerated desktop interface.

## Efficiency Notes

### Strengths

- much faster to develop than raw Win32
- clean modern styling
- built-in layout and resize behavior
- cross-platform direction
- good fit for dashboards, tools, editors, and internal apps

### Tradeoffs

- executable is larger than direct Win32
- more dependencies
- first full release build takes longer
- visual style is framework-driven rather than native Windows controls

## Build

```powershell
cargo build --release
```

Cargo writes the optimized build to:

```text
target/release/modern_ops_studio.exe
```

The prepared demonstration copy is:

```text
dist/ModernOpsStudio.exe
```

## Important Files

```text
Cargo.toml      Package metadata and eframe/egui dependency.
Cargo.lock      Locked Rust package graph.
src/main.rs     Full modern GUI application source.
dist/           Prepared release executable for comparison.
target/         Cargo build output. Safe to delete and ignored by Git.
```
