# wasm-benchmark

Compare host call performance for the same numeric kernel in two WebAssembly shapes:

| Variant | Interface | Host API |
|--------|-----------|----------|
| **Core Wasm** | Exported linear memory + `convert(in_ptr, out_ptr)` | `wasmtime::Memory` + typed func |
| **WIT component** | `export convert: func(values: list<u32>) -> list<f64>` | `wasmtime::component::bindgen!` |

Both guests map 100 `u32` inputs to 100 `f64` outputs using `(v as f64) * 1.5 + sin(i)`.

## Prerequisites

- Rust stable (2021 edition)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- One of:
  - [`cargo-component`](https://github.com/bytecodealliance/cargo-component): `cargo install cargo-component`, or
  - [`wasm-tools`](https://github.com/bytecodealliance/wasm-tools): `cargo install wasm-tools` (used as fallback from `host/build.rs`)

## Build and run benchmarks

```powershell
# Windows
.\scripts\build-guests.ps1
cargo bench -p host
```

```bash
# Linux / macOS
./scripts/build-guests.sh
cargo bench -p host
```

Guest modules are written to `target-guest/` so guest builds do not contend with the host `target/` lock.

`host/build.rs` only checks that guest artifacts exist; it does not invoke nested `cargo` during host compilation.

## Benchmark results (sample)

Measured with Criterion on **Windows 10**, **release** profile, **wasmtime 29**, after `.\scripts\build-guests.ps1` and `cargo bench -p host`. Each iteration converts **100** `u32` → **100** `f64` (host setup + Wasm call + reading results; core path includes writing inputs into guest linear memory).

| Benchmark | Time (per call) | Throughput |
|-----------|-----------------|------------|
| `convert_u32_to_f64/core_wasm_memory` | ~1.21 µs | ~82 Melem/s |
| `convert_u32_to_f64/wit_component` | ~1.77 µs | ~56 Melem/s |

On this run, the **WIT component** path was about **~46% slower** than **core Wasm + in-out memory**, mostly from component-model list marshaling and typed host bindings on top of the same guest kernel. Your numbers will differ by CPU, compiler, and wasmtime version; use `target/criterion/` (or `target-host/criterion/` if you set `CARGO_TARGET_DIR`) for full reports after `cargo bench`.

## Layout

- `wit/convert.wit` — component interface
- `guest-core/` — core module (`convert` + exported memory)
- `guest-component/` — WIT export via `wit-bindgen`
- `host/` — wasmtime loader and Criterion benchmarks

## Core memory layout

The host writes 100 `u32` at offset `0` and reads 100 `f64` at offset `400` (`INPUT_BYTES`), so the guest linear memory only needs `800` bytes for this call pattern.
