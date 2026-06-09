# Rust 公式 devcontainer ベース（cargo / rustfmt / clippy 同梱、CARGO_HOME=/usr/local/cargo）
FROM mcr.microsoft.com/devcontainers/rust:1

# Zed Remote / エディタが使う rust-analyzer と lint 系コンポーネントを固定で入れておく
# （image に rustfmt/clippy は入っているが rust-analyzer は無いことがあるため明示）
USER vscode
RUN rustup component add rust-analyzer clippy rustfmt

# cargo-registry を named volume（別 fs）に逃がすため、target は各プロジェクト内 .gitignore 済みの ./target に置く
# registry の所有権は postCreate(setup.sh) で vscode に直す
