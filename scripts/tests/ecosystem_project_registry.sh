#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

# Targeted only: exercises the fiche/unknown-field round-trip and concurrent
# stale-writer merge without touching ~/.omega/projects.json.
cargo test -p omega-core project_manager::tests::ecosystem_ -- --nocapture
