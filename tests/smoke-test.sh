#!/usr/bin/env bash
set -uo pipefail

BIN="${WEOLINE_BIN:-./target/release/weoline}"
PASS=0
FAIL=0

pass() { echo "  PASS  $1"; ((PASS++)) || true; }
fail() { echo "  FAIL  $1"; ((FAIL++)) || true; }

contains()     { [[ "$1" == *"$2"* ]]; }
not_contains() { [[ "$1" != *"$2"* ]]; }
is_empty()     { [[ -z "$1" ]]; }

if [ ! -f "$BIN" ]; then
  echo "Building release..."
  cargo build --release --quiet
fi

# --- Pipe Mode ---
echo ""
echo "=== Pipe Mode ==="

JSON='{"context_window":{"used_percentage":45,"context_window_size":200000}}'

out=$(echo "$JSON" | "$BIN")
contains "$out" "45%"      && pass "pipe: context bar"     || fail "pipe: context bar"
contains "$out" "90k/200k" && pass "pipe: token counts"    || fail "pipe: token counts"

out=$(echo "$JSON" | SL_MODE=compact "$BIN")
contains "$out" "45%" && pass "pipe: compact mode" || fail "pipe: compact mode"

out=$(echo "$JSON" | SL_MODE=minimal "$BIN")
contains "$out" "45%" && pass "pipe: minimal mode" || fail "pipe: minimal mode"

out=$(echo '{}' | SL_SHOW_LIMITS=0 "$BIN")
is_empty "$out" && pass "pipe: empty input" || fail "pipe: empty input"

out=$(echo 'not json' | SL_SHOW_LIMITS=0 "$BIN")
is_empty "$out" && pass "pipe: invalid json" || fail "pipe: invalid json"

# --- Help ---
echo ""
echo "=== Help ==="

out=$("$BIN" --help)
contains "$out" "USAGE"   && pass "help: shows usage" || fail "help: shows usage"
contains "$out" "--query"  && pass "help: shows query" || fail "help: shows query"
contains "$out" "SL_MODE"  && pass "help: shows env"   || fail "help: shows env"

out=$("$BIN" -h)
contains "$out" "USAGE" && pass "help: short flag" || fail "help: short flag"

# --- Query Mode ---
echo ""
echo "=== Query Mode ==="

if "$BIN" --query -f json 2>/dev/null | head -1 | grep -q '{'; then
  out=$("$BIN" --query -f json)
  echo "$out" | python3 -c "import sys,json; json.load(sys.stdin)" 2>/dev/null \
    && pass "query: json valid"          || fail "query: json valid"
  contains "$out" '"meta"'       && pass "query: json has meta"      || fail "query: json has meta"
  contains "$out" '"five_hour"'  && pass "query: json has five_hour" || fail "query: json has five_hour"

  out=$("$BIN" --query -f json -d minimal)
  contains "$out" "five_hour_pct"    && pass "query: minimal has pct" || fail "query: minimal has pct"
  not_contains "$out" '"meta"'       && pass "query: minimal no meta" || fail "query: minimal no meta"

  out=$("$BIN" --query -f json --filter sonnet)
  not_contains "$out" '"five_hour"'    && pass "query: filter excludes five_hour" || fail "query: filter excludes five_hour"
  contains "$out" "seven_day_sonnet"   && pass "query: filter includes sonnet"    || fail "query: filter includes sonnet"

  out=$("$BIN" --query -f json -d minimal --filter five-hour)
  contains "$out" "five_hour_pct"        && pass "query: minimal+filter has 5h"   || fail "query: minimal+filter has 5h"
  not_contains "$out" "seven_day_pct"    && pass "query: minimal+filter no 7d"    || fail "query: minimal+filter no 7d"

  out=$("$BIN" --query)
  contains "$out" "5h:"    && pass "query: toon has 5h"     || fail "query: toon has 5h"
  contains "$out" "7d:"    && pass "query: toon has 7d"     || fail "query: toon has 7d"
  contains "$out" "sonnet:" && pass "query: toon has sonnet" || fail "query: toon has sonnet"

  out=$("$BIN" --query -d minimal)
  not_contains "$out" "⏱"  && pass "query: toon minimal no emoji" || fail "query: toon minimal no emoji"
  contains "$out" "5h:"    && pass "query: toon minimal has pct"  || fail "query: toon minimal has pct"
else
  echo "  SKIP  query tests (no cache data available)"
fi

# --- Error Cases ---
echo ""
echo "=== Error Cases ==="

SL_CACHE_FILE=/tmp/nonexistent-weoline-cache "$BIN" --query 2>/dev/null
[[ $? -eq 1 ]] && pass "query: missing cache exits 1" || fail "query: missing cache exits 1"

# --- Results ---
echo ""
echo "=== Results ==="
echo "  $PASS passed, $FAIL failed"
[[ "$FAIL" -eq 0 ]]
