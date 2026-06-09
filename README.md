# underrated

Gecko / WebKit / Blink に頼らない、ゼロから自前実装する Web ブラウザエンジン（Rust, edition 2024）。

## セットアップ

DevContainer（Zed Remote / VS Code Dev Containers）で開くと `postCreate` が走り、`cargo fetch` まで完了する。

## 使い方

```bash
cargo run                    # 実行
cargo test                   # テスト
cargo clippy --all-targets   # lint
cargo fmt                    # format
```

## ライセンス

[Apache-2.0](LICENSE)
