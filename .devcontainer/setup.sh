#!/usr/bin/env bash
set -euo pipefail

# A named volume is initially created by the Docker Engine as root, so fix ownership to vscode.
# Important: chown not only the mount point but also its **parent directory** (lesson from
#   _template-python/setup.sh: if .cache/.config are left owned by root, the Zed Remote Server
#   dies silently after 60s with "Permission denied creating .cache/zed").
sudo chown vscode:vscode \
  /home/vscode/.config \
  /home/vscode/.cache 2>/dev/null || true
sudo chown -R vscode:vscode \
  /home/vscode/.config/gh \
  /home/vscode/.claude \
  /home/vscode/.gemini 2>/dev/null || true
# Make the cargo-registry volume (under CARGO_HOME) vscode-owned too.
sudo chown -R vscode:vscode \
  /usr/local/cargo/registry 2>/dev/null || true

# Install the Gemini CLI (the Zed Agent Panel is expected to launch it via ACP inside this project).
# The node feature is installed first, so npm is available at this postCreate point.
if command -v npm >/dev/null 2>&1 && ! command -v gemini >/dev/null 2>&1; then
    echo "→ npm install -g @google/gemini-cli"
    npm install -g @google/gemini-cli || echo "  (gemini CLI install failed; try 'npm i -g @google/gemini-cli' manually)"
fi

# git configuration (same policy as _template-python).
git config --global core.autocrlf input
git config --global init.defaultBranch main
git config --global url."https://github.com/".insteadOf "git@github.com:"
git config --global pull.rebase false

# If a Cargo.toml exists, build once to resolve dependencies (target is cached under ./target).
if [ -f "Cargo.toml" ]; then
    echo "→ cargo fetch"
    cargo fetch
fi

cat <<'EOF'

================================================================
Rust DevContainer (toy-browser) initialization complete.

[First-time setup]
1. GitHub auth:
     gh auth login
2. Claude Code auth (if already /login'd in another container, shared via volume):
     claude /login
3. Gemini auth (from the Zed Agent Panel or terminal; persisted to shared-gemini-config):
     gemini   # Google login on first run; afterwards authenticated via the shared volume
4. Smoke check:
     cargo run

[Day-to-day]
  cargo add <crate>            # add a dependency (updates Cargo.toml + Cargo.lock)
  cargo run                    # run
  cargo test                   # test
  cargo clippy --all-targets   # lint
  cargo fmt                    # format
  cargo build --release        # release build

[Local LLM (llama-server) check]
  curl http://llama-server:8080/v1/models
================================================================
EOF
