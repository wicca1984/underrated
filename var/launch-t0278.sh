#!/usr/bin/env bash
# Launcher for Gemini worker t0278 — setsid-detached so it survives the orchestrator tick.
set -euo pipefail
WT=/workspaces/wt/t0278
PROMPT_FILE=/workspaces/toy-browser/var/prompts/t0278.txt
LOG=/workspaces/toy-browser/var/worker-logs/t0278.log

# Make GEMINI_API_KEY available (canonical source: var/.env, else ~/.bashrc, else inherited env).
if [ -f /workspaces/toy-browser/var/.env ]; then
  set -a; . /workspaces/toy-browser/var/.env; set +a
elif [ -z "${GEMINI_API_KEY:-}" ] && [ -f "$HOME/.bashrc" ]; then
  # shellcheck disable=SC1090
  . "$HOME/.bashrc" || true
fi

cd "$WT"
PROMPT="$(cat "$PROMPT_FILE")"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null >> "$LOG" 2>&1
