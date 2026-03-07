#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR"

echo "[g2] verify scene flow"
cargo test --manifest-path logic/Cargo.toml title_pause_result_and_exit_flow_is_reachable -- --nocapture

echo "[g2] verify 30-minute headless stability harness"
cargo test --manifest-path logic/Cargo.toml headless_g2_stability_run_reaches_clear_with_safe_corridor -- --nocapture

echo "[g2] complete"
