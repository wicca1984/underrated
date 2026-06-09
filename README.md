<p align="center">
  <img src="img/Underrated_logo.png" alt="underrated" width="320">
</p>

# underrated

An independent web browser engine, written from scratch in Rust (edition 2024) — no Gecko, WebKit, or Blink.

> **Status: early development.** `underrated` is a rendering **engine** being built one stage at a time
> (HTML parsing → DOM → CSS → layout → paint). It is **not usable as a browser yet** — there is no
> window and nothing is drawn; running it today only prints a placeholder line. This README is for
> people who want to **build the engine from source** and follow or contribute to its development.

## Prerequisites

- A Rust toolchain for **edition 2024 (Rust 1.85 or newer)**. The channel is pinned in
  [`rust-toolchain.toml`](rust-toolchain.toml), so with [`rustup`](https://rustup.rs) installed the
  correct toolchain (plus `rustfmt` and `clippy`) is selected automatically.
- `git` (the test suite uses a submodule of conformance test data).

## Building from source

### Option A — Dev Container (recommended)

Open the repository in a Dev Container (VS Code Dev Containers or Zed Remote). `postCreate`
provisions the toolchain and runs `cargo fetch`. Then build:

```bash
cargo build
```

### Option B — Local checkout

```bash
git clone --recurse-submodules https://github.com/wicca1984/underrated
cd underrated
cargo build
```

> `--recurse-submodules` pulls the [html5lib-tests](https://github.com/html5lib/html5lib-tests)
> data used by the parser tests. If you already cloned without it:
> `git submodule update --init --recursive`.

## Running

```bash
cargo run
```

At this stage there is **no rendered output**: the binary prints a placeholder and exits. Real
browsing (a window and painted pages) is a later milestone.

## Development

The same checks gate every push and pull request in CI:

```bash
cargo test                                  # unit + integration tests
cargo clippy --all-targets -- -D warnings   # lint (warnings are errors)
cargo fmt --all --check                     # formatting check
```

`unsafe` code is forbidden crate-wide, and `unwrap`/`expect` are denied in non-test code — these are
enforced by the compiler and CI, not by convention.

## License

[Apache-2.0](LICENSE)
