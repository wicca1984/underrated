#!/usr/bin/env bash
# Launcher for Gemini worker t0269 — setsid-detached so it survives the orchestrator tick.
set -euo pipefail
WT=/workspaces/wt/t0269
PROMPT_FILE=/workspaces/toy-browser/var/prompts/t0269.txt
LOG=/workspaces/toy-browser/var/worker-logs/t0269.log

# Make GEMINI_API_KEY available (canonical source: var/.env, else inherited env).
if [ -f /workspaces/toy-browser/var/.env ]; then
  set -a; . /workspaces/toy-browser/var/.env; set +a
fi

cd "$WT"
PROMPT="$(cat "$PROMPT_FILE")"
exec gemini -p "$PROMPT" \
  -m gemini-3.5-flash \
  --approval-mode yolo \
  -o stream-json \
  --include-directories /workspaces/underrated-meta \
  < /dev/null
