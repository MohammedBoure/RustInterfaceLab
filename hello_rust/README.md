# Hello Rust

This is a small Rust console program used as a toolchain sanity check.

## Purpose

Before building desktop GUI applications, this project confirms that:

- `rustc` works
- `cargo` works
- a Rust binary can be compiled and run locally

It is not part of the GUI comparison except as a baseline that proves the Rust environment is ready.

## Run

```powershell
cargo run
```

Expected output:

```text
Rust is ready.
Numbers: [3, 7, 11, 19]
Total: 40
```

## Important Files

```text
Cargo.toml      Package metadata. No third-party dependencies.
Cargo.lock      Locked Rust package graph.
src/main.rs     Simple Rust console source.
target/         Cargo build output. Safe to delete and ignored by Git.
```
