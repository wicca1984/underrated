#!/usr/bin/env bash
set -euo pipefail

# named volume は Docker Engine が root で初期作成するため、所有権を vscode に修正。
# 重要: mount 先だけでなく **親ディレクトリ** も chown する（_template-python/setup.sh の教訓。
#   .cache/.config を root 所有のまま放置すると Zed Remote Server が
#   「Permission denied creating .cache/zed」で 60秒後に静かに死ぬ）。
sudo chown vscode:vscode \
  /home/vscode/.config \
  /home/vscode/.cache 2>/dev/null || true
sudo chown -R vscode:vscode \
  /home/vscode/.config/gh \
  /home/vscode/.claude \
  /home/vscode/.gemini 2>/dev/null || true
# cargo-registry volume（CARGO_HOME 配下）も vscode 所有に
sudo chown -R vscode:vscode \
  /usr/local/cargo/registry 2>/dev/null || true

# Gemini CLI を導入（Zed Agent Panel が ACP 経由でこのプロジェクト環境内で起動する想定）。
# node feature が先に入っているので postCreate のこの時点では npm が使える。
if command -v npm >/dev/null 2>&1 && ! command -v gemini >/dev/null 2>&1; then
    echo "→ npm install -g @google/gemini-cli"
    npm install -g @google/gemini-cli || echo "  (gemini CLI のインストールに失敗。手動で npm i -g @google/gemini-cli を試す)"
fi

# git 設定（_template-python と同方針）
git config --global core.autocrlf input
git config --global init.defaultBranch main
git config --global url."https://github.com/".insteadOf "git@github.com:"
git config --global pull.rebase false

# Cargo.toml があれば一度ビルドして依存を解決（target は ./target にキャッシュ）
if [ -f "Cargo.toml" ]; then
    echo "→ cargo fetch"
    cargo fetch
fi

cat <<'EOF'

================================================================
Rust DevContainer (toy-browser) 初期化が完了しました。

【初回セットアップ】
1. GitHub 認証:
     gh auth login
2. Claude Code 認証（既に他コンテナで /login 済なら共有 volume 経由で認証済）:
     claude /login
3. Gemini 認証（Zed Agent Panel から or ターミナルで。shared-gemini-config に永続化）:
     gemini   # 初回に Google ログイン。以降は共有 volume で認証済
4. 動作確認:
     cargo run

【日々の運用】
  cargo add <crate>            # 依存追加（Cargo.toml + Cargo.lock 更新）
  cargo run                    # 実行
  cargo test                   # テスト
  cargo clippy --all-targets   # lint
  cargo fmt                    # format
  cargo build --release        # リリースビルド

【ローカル LLM (llama-server) 確認】
  curl http://llama-server:8080/v1/models
================================================================
EOF
