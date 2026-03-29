#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_FILE="${SCRIPT_DIR}/ruby.pine"

# Fallback: look in the sibling ruby directory
if [ ! -f "$TEST_FILE" ]; then
    TEST_FILE="${ROOT_DIR}/../ruby/ruby.pine"
fi

if [ ! -f "$TEST_FILE" ]; then
    echo "ERROR: Cannot find ruby.pine test file"
    echo "  Expected at: ${SCRIPT_DIR}/ruby.pine"
    echo "  Or at:       ${ROOT_DIR}/../ruby/ruby.pine"
    exit 1
fi

echo "═══════════════════════════════════════════════"
echo "  pine-lsp test runner"
echo "═══════════════════════════════════════════════"
echo ""

# Step 1: Build pine-lsp (suppress warnings, only show errors)
echo "▸ Building pine-lsp..."
cd "$ROOT_DIR"
if cargo build -p pine-lsp 2>&1 | grep "^error" ; then
    echo "  ✗ Build failed"
    exit 1
fi
echo "  ✓ Build succeeded"
echo ""

# Step 2: Run the parser/linter against ruby.pine
LINE_COUNT="$(wc -l < "$TEST_FILE" | tr -d ' ')"
echo "▸ Linting: $(basename "$TEST_FILE") (${LINE_COUNT} lines)"
cargo run --quiet -p pine-lsp -- --check "$TEST_FILE" 2>/dev/null
echo ""

# Step 3: Run unit tests (suppress warnings, only show test results)
echo "▸ Running unit tests..."
cargo test --quiet -p pine-lsp 2>/dev/null
echo ""

echo "═══════════════════════════════════════════════"
echo "  Done."
echo "═══════════════════════════════════════════════"
