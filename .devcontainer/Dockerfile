# Official Rust devcontainer base (bundles cargo / rustfmt / clippy, CARGO_HOME=/usr/local/cargo)
FROM mcr.microsoft.com/devcontainers/rust:1

# Pin the rust-analyzer and lint components used by Zed Remote / the editor.
# (the image ships rustfmt/clippy, but rust-analyzer is sometimes missing, so add it explicitly)
USER vscode
RUN rustup component add rust-analyzer clippy rustfmt

# Since cargo-registry is offloaded to a named volume (separate fs), target lives in each
# project's ./target (already .gitignore'd).
# The registry's ownership is fixed to vscode in postCreate (setup.sh).
