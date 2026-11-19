#!/usr/bin/env bash
#
# Response-parity check: runs the same requests against the Laravel app and
# the Rust rewrite and diffs the JSON.
#
# Requirements:
#   - Laravel app serving a COPY of the SQLite dataset (so cache/log rows
#     written by the run never touch the repository's DB file):
#
#       cp database/database.sqlite /tmp/openalbion-parity.sqlite
#       sqlite3 /tmp/openalbion-parity.sqlite "INSERT INTO api_tokens
#         (user_id, name, token, created_at, updated_at)
#         VALUES (1, 'parity', 'parity-token', datetime('now'), datetime('now'));"
#       cd public && DB_DATABASE=/tmp/openalbion-parity.sqlite \
#         WEAPONRY_KEY=paritykey123 php -S 127.0.0.1:8000 \
#         ../vendor/laravel/framework/src/Illuminate/Foundation/resources/server.php
#
#   - Rust app serving the exported JSON data (default http://127.0.0.1:9911).
#     The `WEAPONRY_KEY` env var below is only for the *Laravel* side — the
#     Rust app is fully public and just ignores the header:
#
#       cd rust && DATA_DIR=./data \
#         RATE_LIMIT_PER_MINUTE=1000000 PORT=9911 ./target/release/openalbion-api
#
#   - jq and python3 on PATH
#
# Usage: LARAVEL_DB=/tmp/openalbion-parity.sqlite scripts/parity_check.sh

set -u

LARAVEL="${1:-http://127.0.0.1:8000}"
RUST="${2:-http://127.0.0.1:9911}"
WEAPONRY_KEY="${WEAPONRY_KEY:-paritykey123}"
API_TOKEN="${API_TOKEN:-parity-token}"
# SQLite DB the Laravel server is running against. Point this at a COPY of
# database/database.sqlite (via DB_DATABASE when starting the Laravel server)
# so parity runs never write cache/log rows into the repository's DB file.
LARAVEL_DB="${LARAVEL_DB:-$(cd "$(dirname "$0")/../.." && pwd)/database/database.sqlite}"

PASS=0
FAIL=0
FAILED_PATHS=()

# Laravel throttles at 60 req/min per IP; wipe its rate-limit/response cache
# rows periodically to stay under it. Deleting the `cache` table works no
# matter which DB the server points at.
laravel_cache_clears=0
clear_laravel_cache() {
    laravel_cache_clears=$((laravel_cache_clears + 1))
    python3 - "$LARAVEL_DB" <<'PYEOF' 2>/dev/null || true
import sqlite3, sys
con = sqlite3.connect(sys.argv[1])
con.execute("DELETE FROM cache")
con.commit()
PYEOF
}

maybe_clear_laravel_cache() {
    laravel_cache_clears=$((laravel_cache_clears + 1))
    if [ $((laravel_cache_clears % 30)) -eq 0 ]; then
        clear_laravel_cache
    fi
}

# fetch <base> <path> <auth-mode: none|token|weaponry>
fetch() {
    local base="$1" path="$2" mode="$3"
    local args=(-s -m 20)
    case "$mode" in
        token)   args+=(-H "Authorization: ${API_TOKEN}") ;;
        weaponry) args+=(-H "X-WEAPONRY-KEY: ${WEAPONRY_KEY}") ;;
    esac
    curl "${args[@]}" -w '\n%{http_code}' "${base}${path}"
}

# compare <path> <auth-mode> — diffs status code and normalized JSON body.
# A Laravel 500 caused by the (unconfigured) Algolia engine is reported as a
# known deviation: the Rust rewrite replaced weaponryV2 search with an
# in-memory search and does not depend on Algolia.
compare() {
    local path="$1" mode="${2:-none}"

    maybe_clear_laravel_cache
    local php_raw rust_raw php_body php_code rust_body rust_code
    php_raw="$(fetch "$LARAVEL" "$path" "$mode")"
    php_code="${php_raw##*$'\n'}"
    php_body="${php_raw%$'\n'*}"

    if [ "$php_code" = "429" ]; then
        clear_laravel_cache
        php_raw="$(fetch "$LARAVEL" "$path" "$mode")"
        php_code="${php_raw##*$'\n'}"
        php_body="${php_raw%$'\n'*}"
    fi

    if [ "$php_code" = "500" ] && printf '%s' "$php_body" | grep -q Algolia; then
        echo "SKIP  $path — Laravel 500 (Algolia not configured); replaced by in-memory search in Rust (known deviation)"
        return 0
    fi

    rust_raw="$(fetch "$RUST" "$path" "$mode")"
    rust_code="${rust_raw##*$'\n'}"
    rust_body="${rust_raw%$'\n'*}"

    local php_norm rust_norm
    php_norm="$(printf '%s' "$php_body" | jq -S . 2>/dev/null || printf '%s' "$php_body")"
    rust_norm="$(printf '%s' "$rust_body" | jq -S . 2>/dev/null || printf '%s' "$rust_body")"

    if [ "$php_code" = "$rust_code" ] && [ "$php_norm" = "$rust_norm" ]; then
        PASS=$((PASS + 1))
        printf 'OK    [%s] %s\n' "$php_code" "$path"
    else
        FAIL=$((FAIL + 1))
        FAILED_PATHS+=("$path")
        printf 'DIFF  %s\n' "$path"
        printf '      laravel=%s rust=%s\n' "$php_code" "$rust_code"
        diff <(printf '%s\n' "$php_norm") <(printf '%s\n' "$rust_norm") | head -12 | sed 's/^/      /'
    fi
}

echo "Laravel: $LARAVEL"
echo "Rust:    $RUST"
echo

# ---------------------------------------------------------------------------
# v1 (token-protected in Laravel; public by decision in Rust)
# ---------------------------------------------------------------------------
compare "/api/v1/categories" token
compare "/api/v1/categories?type=weapon" token
compare "/api/v1/categories?type=mount" token
compare "/api/v1/weapons" token
compare "/api/v1/weapons?category_id=1" token
compare "/api/v1/weapons?subcategory_id=2" token
compare "/api/v1/weapons?tier=4" token
compare "/api/v1/weapons?tier=0" token
compare "/api/v1/weapons?category_id=abc" token
compare "/api/v1/armors" token
compare "/api/v1/accessories" token
compare "/api/v1/spells/weapon/1" token
compare "/api/v1/spells/armor/1" token
compare "/api/v1/spells/accessory/1" token
compare "/api/v1/weapon-stats/weapon/1" token
compare "/api/v1/armor-stats/armor/1" token
compare "/api/v1/accessory-stats/accessory/1" token

# ---------------------------------------------------------------------------
# v2 (token-protected in Laravel)
# ---------------------------------------------------------------------------
compare "/api/v2/categories" token
compare "/api/v2/categories?type=consumable" token
compare "/api/v2/weapons?tier=8" token
compare "/api/v2/armors" token
compare "/api/v2/accessories" token
compare "/api/v2/consumables" token
compare "/api/v2/consumables?category_id=5" token
compare "/api/v2/spells/weapon/1" token
compare "/api/v2/weapon-stats/weapon/1" token
compare "/api/v2/consumable-stats/consumable/1" token
compare "/api/v2/consumable-craftings/consumable/1" token

# ---------------------------------------------------------------------------
# v3 (public in both)
# ---------------------------------------------------------------------------
compare "/api/v3/categories"
compare "/api/v3/categories?type=armor"
compare "/api/v3/weapons"
compare "/api/v3/weapons?category_id=1&tier=4"
compare "/api/v3/weapons?tier=4.0"
compare "/api/v3/weapons?tier=abc"
compare "/api/v3/armors"
compare "/api/v3/accessories"
compare "/api/v3/consumables"
compare "/api/v3/spells/weapon/1"
compare "/api/v3/spells/armor/1"
compare "/api/v3/spells/accessory/1"
compare "/api/v3/weapon-stats/weapon/1"
compare "/api/v3/armor-stats/armor/1"
compare "/api/v3/accessory-stats/accessory/1"
compare "/api/v3/consumable-stats/consumable/1"
compare "/api/v3/consumable-craftings/consumable/1"
compare "/api/v3/spells/weapon/999999"
compare "/api/v3/weapon-stats/weapon/999999"
compare "/api/v3/consumable-stats/consumable/999999"
compare "/api/v3/spells/weapon/abc"
compare "/api/v3/weapons/"

# ---------------------------------------------------------------------------
# weaponry (v1 route set) + weaponryV2 (v2 route set)
# ---------------------------------------------------------------------------
compare "/api/weaponry/version-check" weaponry
compare "/api/weaponry/search?search=sword" weaponry
compare "/api/weaponry/search?search=SWORD" weaponry
compare "/api/weaponry/search" weaponry
compare "/api/weaponry/categories" weaponry
compare "/api/weaponry/categories?type=armor" weaponry
compare "/api/weaponry/weapons?tier=4" weaponry
compare "/api/weaponry/spells/weapon/1" weaponry
compare "/api/weaponry/weapon-stats/weapon/1" weaponry

compare "/api/weaponryV2/version-check" weaponry
compare "/api/weaponryV2/search?search=axe" weaponry
compare "/api/weaponryV2/categories" weaponry
compare "/api/weaponryV2/consumables" weaponry
compare "/api/weaponryV2/consumable-craftings/consumable/1" weaponry

# ---------------------------------------------------------------------------
# Error shape: unknown route. Both must answer 404; Laravel serves its HTML
# error page while the Rust app answers JSON (documented deviation), so only
# the status code is compared here.
# ---------------------------------------------------------------------------
maybe_clear_laravel_cache
php_status="$(fetch "$LARAVEL" "/api/v3/unknown" none | tail -n1)"
rust_status="$(fetch "$RUST" "/api/v3/unknown" none | tail -n1)"
if [ "$php_status" = "$rust_status" ]; then
    PASS=$((PASS + 1))
    printf 'OK    [%s] /api/v3/unknown (status only — body differs by design)\n' "$php_status"
else
    FAIL=$((FAIL + 1))
    FAILED_PATHS+=("/api/v3/unknown")
    printf 'DIFF  /api/v3/unknown laravel=%s rust=%s\n' "$php_status" "$rust_status"
fi

# ---------------------------------------------------------------------------
# AOD price proxy: live external data — compare the response SHAPE only.
# Clear Laravel's cache first so a previously-cached upstream timeout
# (cached for 3600s by Laravel) does not poison the comparison.
# ---------------------------------------------------------------------------
clear_laravel_cache
aod_shape() {
    local base="$1"
    curl -s -m 10 -H "X-WEAPONRY-KEY: ${WEAPONRY_KEY}" \
        "${base}/api/weaponry/aod/west/item/T4_BAG/price?locations=Caerleon" \
        | jq -S 'if type == "array" then {type: "array", keys: (map(keys) | add | unique)} else . end' 2>/dev/null
}
php_aod="$(aod_shape "$LARAVEL")"
rust_aod="$(aod_shape "$RUST")"
if [ -n "$php_aod" ] && [ "$php_aod" = "$rust_aod" ]; then
    PASS=$((PASS + 1))
    echo "OK    [shape] /api/weaponry/aod/west/item/T4_BAG/price"
else
    echo "INFO  AOD price shape differs or upstream unreachable (live data) — showing both:"
    echo "      laravel: ${php_aod:0:200}"
    echo "      rust:    ${rust_aod:0:200}"
fi

# ---------------------------------------------------------------------------
# Web pages: the Rust UI is a deliberate redesign, so text/HTML no longer
# mirrors the Laravel blade views. Only the status code is compared.
# ---------------------------------------------------------------------------
for path in "/" "/terms-of-service" "/privacy-policy"; do
    php_code="$(curl -s -o /dev/null -m 8 -w '%{http_code}' "$LARAVEL$path")"
    rust_code="$(curl -s -o /dev/null -m 8 -w '%{http_code}' "$RUST$path")"
    if [ "$php_code" = "$rust_code" ] && [ "$rust_code" = "200" ]; then
        PASS=$((PASS + 1))
        printf 'OK    [%s] %s (status only — UI intentionally redesigned)\n' "$rust_code" "$path"
    else
        FAIL=$((FAIL + 1))
        FAILED_PATHS+=("web:$path")
        printf 'DIFF  %s laravel=%s rust=%s\n' "$path" "$php_code" "$rust_code"
    fi
done

# /download redirect: identical status and target
dl_laravel="$(curl -s -o /dev/null -m 8 -w '%{http_code} %{redirect_url}' "$LARAVEL/download")"
dl_rust="$(curl -s -o /dev/null -m 8 -w '%{http_code} %{redirect_url}' "$RUST/download")"
if [ "$dl_laravel" = "$dl_rust" ]; then
    PASS=$((PASS + 1))
    echo "OK    [redirect] /download -> $dl_laravel"
else
    FAIL=$((FAIL + 1))
    FAILED_PATHS+=("web:/download")
    echo "DIFF  [redirect] /download"
    echo "      laravel: $dl_laravel"
    echo "      rust:    $dl_rust"
fi

# robots.txt (served from the same source file in both apps)
if [ "$(curl -s "$LARAVEL/robots.txt")" = "$(curl -s "$RUST/robots.txt")" ]; then
    PASS=$((PASS + 1))
    echo "OK    [text] /robots.txt"
else
    FAIL=$((FAIL + 1))
    FAILED_PATHS+=("web:/robots.txt")
    echo "DIFF  [text] /robots.txt"
fi

# ---------------------------------------------------------------------------
echo
echo "======================================"
echo "passed: $PASS   failed: $FAIL"
if [ "$FAIL" -gt 0 ]; then
    printf 'failing paths:\n'
    printf '  - %s\n' "${FAILED_PATHS[@]}"
    exit 1
fi
