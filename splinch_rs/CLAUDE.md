# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

- **Build:** `cargo build`
- **Run (split):** `cargo run -- -i <file> [-v]`
- **Run (combine):** `cargo run -- -i <file.xor1> -c`
- **Release build:** `cargo build --release`
- **Check (fast compile check):** `cargo check`
- **Run tests:** `cargo test`
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`
- **Build .deb package:** `cargo deb`
- **Preview man page:** `man target/debug/build/splinch_rs-*/out/splinch.1`

No tests exist yet. The project uses Rust 2024 edition.

## Architecture

**splinch_rs** is a CLI tool that splits a file into two XOR-complementary parts for secure transport. Neither output file alone reveals any information about the original; XOR-ing them together reconstructs it (one-time pad splitting).

### Two-file structure

- **`src/lib.rs`** — Core logic: `split_file()`, `verify_files()`, `combine_files()`, and `xor_buffers()`. Splitting generates a random byte stream (xor1) and XORs it with the input to produce xor2. Combining XORs the two parts back together to reconstruct the original. Verification has two strategies: full byte-by-byte comparison for files ≤10MB (`VERIFY_FULL_THRESHOLD`), and sampled verification (10 random 64KB chunks) for larger files.
- **`src/main.rs`** — CLI layer using clap derive. Parses `-i`/`--input`, `-v`/`--verify`, and `-c`/`--combine` flags, calls into lib.

### Key constants (lib.rs)

- `CHUNK_SIZE`: 64KB — processing buffer size for streaming reads/writes
- `VERIFY_FULL_THRESHOLD`: 10MB — files larger than this use sampled verification

### Dependencies

- `clap` (derive) — CLI argument parsing
- `anyhow` — error handling
- `rand` — cryptographic random byte generation for the XOR pad
