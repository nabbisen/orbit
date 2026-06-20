#!/usr/bin/env bash
# check-design-tokens.sh — RFC-032 design-token gate.
#
# Fails if any orbok-ui view/component module contains a literal font size,
# padding, array padding, non-zero spacing, or hard-coded iced colour. The only
# sanctioned styling path is the Snora Design token bridge via `crate::theme`
# helpers and `tokens.spacing.*` (cf. the snora lucide/token gateway rule).
#
# Heuristic, like the RFC-031 string-literal gate: greps text, no parsing.
set -euo pipefail

cd "$(dirname "$0")/.."

# View/component modules that must be fully token-driven.
FILES=$(git ls-files 'crates/ui/src/views.rs' 'crates/ui/src/views/*.rs' \
                     'crates/ui/src/shell.rs' 'crates/ui/src/components.rs' \
                     2>/dev/null || true)
# Tolerate components.rs not existing yet (arrives in RFC-033).
[ -z "$FILES" ] && FILES=$(ls crates/ui/src/views.rs crates/ui/src/views/*.rs \
                              crates/ui/src/shell.rs 2>/dev/null || true)

fail=0
flag() { echo "design-token gate: $1"; fail=1; }

# Literal text sizes: .size(12)   (allow .size(theme::...), .size(var))
if grep -nE '\.size\([0-9]' $FILES; then flag "literal text size — use theme::{body,meta,...}"; fi
# Literal bare paddings: .padding(10)
if grep -nE '\.padding\([0-9]' $FILES; then flag "literal padding — use tokens.spacing.*"; fi
# Literal array paddings: Padding::from([12.0, 16.0])
if grep -nE 'Padding::from\(\[[0-9.]' $FILES; then flag "literal array padding — use tokens.spacing.*"; fi
# Non-zero literal spacing: .spacing(8)   (spacing(0) is an allowed structural zero)
if grep -nE '\.spacing\([1-9]' $FILES; then flag "literal spacing — use tokens.spacing.*"; fi
# Hard-coded colours.
if grep -nE 'iced::Color|Color::from_rgb|from_rgba' $FILES; then flag "literal colour — use palette roles via the token bridge"; fi

if [ "$fail" -ne 0 ]; then
  echo "FAIL: magic styling values found in view modules (RFC-032)."
  exit 1
fi
echo "design-token gate: ok"
