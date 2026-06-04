# Modern Ops Spreadsheet

This project is a modern spreadsheet-style Rust desktop application built with `eframe/egui`.

## Purpose

This app demonstrates how far a Rust-native immediate-mode GUI can be pushed toward an Excel-like desktop program without introducing a dedicated spreadsheet engine.

The goal is to test `eframe/egui` as a serious desktop application toolkit: dense grids, editing, formulas, formatting, data operations, side panels, and CSV movement.

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

- 80 x 26 spreadsheet grid
- A1-style cell addressing
- formula bar
- range box such as `A1:D12`
- formulas: `SUM`, `AVG`, `AVERAGE`, `MIN`, `MAX`, `COUNT`, `ABS`, `ROUND`
- expanded formulas: `IF`, `SUMIF`, `COUNTIF`, `VLOOKUP`, `XLOOKUP`, `MEDIAN`, `STDEV`, `VAR`, `CORREL`, `ROUNDUP`
- advanced analysis formulas: `PERCENTILE`, `QUARTILE`, `MODE`, `COVARIANCE`
- arithmetic formulas with `+`, `-`, `*`, `/`, parentheses, and cell references
- range formulas such as `=SUM(C2:C6)`
- circular reference detection
- dependency graph with cached values and dirty dependent recalculation
- typed values for numbers, text, booleans, dates, currency, and percentages
- Excel-like errors such as `#REF!`, `#VALUE!`, `#DIV/0!`, `#NAME?`, `#N/A`, and `#CYCLE!`
- number, currency, and percent formatting
- cell fill colors, bold, italic, and alignment
- sample workbook with realistic operational data
- selection statistics: sum, average, min, max, numeric count, text count, errors
- expanded summary panel with standard deviation and P25/P50/P75 percentiles
- charts for selected data: bar, line, pie, and scatter
- grouped summaries using the first selected column as a category and the second as values
- auto-filter controls by column text
- conditional formatting rules over selected ranges
- data validation rules for numeric ranges, dates, lists, and required cells
- internal copy/paste using tab-separated values
- fill-down operation
- sort selected ranges
- find and replace inside a selected range
- named range registry
- CSV export/import text buffer
- show-formulas mode
- zoom control

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
- good fit for dashboards, tools, spreadsheet-like internal apps, editors, and dense operations tools

### Tradeoffs

- executable is larger than direct Win32
- more dependencies
- first full release build takes longer
- visual style is framework-driven rather than native Windows controls
- large Excel-class behavior still requires a dedicated data model, formula engine, file format support, and extensive optimization

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
