# OpenAlbion API (Rust)

The OpenAlbion API rewritten in Rust — a single binary serving the same
endpoints as the Laravel app, but reading the dataset from **JSON files
instead of a database**.

Serves:

- `/api/v1`, `/api/v2`, `/api/v3` — the public data API (v1/v2 no longer
  require an API token; v3 never did)
- `/api/weaponry`, `/api/weaponryV2` — the Weaponry mobile-app endpoints,
  each including its full version route set (**fully public** — no
  `X-WEAPONRY-KEY` check)
- `/`, `/terms-of-service`, `/privacy-policy`, `/download` — the web pages

## Quick start

```bash
cd rust

# 1. Generate the data files from the existing SQLite database
#    (already committed under rust/data — only needed when the DB changes)
python3 scripts/export_sqlite.py ../database/database.sqlite data

# 2. Run (reads ./data by default)
cargo run
# OpenAlbion API listening on http://0.0.0.0:8080
```

The dataset (~30k rows, ~10 MB of JSON) is loaded into memory at startup, so
every request is served from RAM with no per-request I/O.

## Configuration (environment or `.env`)

| Variable                  | Default                      | Meaning                                              |
|---------------------------|------------------------------|------------------------------------------------------|
| `HOST`                    | `0.0.0.0`                    | Bind address                                         |
| `PORT`                    | `8080`                       | Listen port                                          |
| `DATA_DIR`                | `data`                       | Directory holding the exported JSON tables           |
| `SLACK_WEBHOOK_URL`       | —                            | Bug reports are forwarded to this Slack webhook      |
| `RATE_LIMIT_PER_MINUTE`   | `60`                         | Requests per minute per IP across all `/api/*`      |
| `AOD_CACHE_SECONDS`       | `180`                        | TTL for the Albion Online Data price proxy           |
| `AOD_URL_TEMPLATE`        | the real AOD URL             | `{region}` and `{item}` placeholders (mainly tests)  |
| `RUST_LOG`                | `info`                       | Log filter                                          |

Bug reports are always appended to `DATA_DIR/bug-reports.jsonl`, whether or
not Slack is configured.

## Data files

One JSON file per table under `DATA_DIR` (see
`scripts/export_sqlite.py`):

```
categories  weapons  armors  accessories  consumables  spells
weapon_spells  armor_spells  accessory_spells
weapon_stats  armor_stats  accessory_stats  consumable_stats
consumable_craftings
```

To update the dataset: refresh `database/database.sqlite` from the Laravel
side, then re-run the export script and restart the app.

The Albion Online Wiki's bot protection is respected: **no crawler or
Cloudflare bypass ships with this project**. Item data is refreshed from the
public game-data dump instead (see below).

## Refreshing item data (`openalbion-import`)

[ao-bin-dumps](https://github.com/ao-data/ao-bin-dumps) publishes the game's
own item definitions (tier, item power, shop categories, crafting recipes,
enchantment variants, localized names). It is the **authoritative** item
source: the importer overwrites the item tables with it.

```bash
cargo run --release --features import --bin openalbion-import --   --dry-run                # inspect the diff first (writes nothing)
cargo run --release --features import --bin openalbion-import      # apply
```

What happens on every run:

1. the current dataset (all 14 tables) is copied to
   `data/backups/<UTC timestamp>/` — that is the version history, so every
   overwrite is reversible;
2. `weapons`, `armors`, `accessories`, `consumables` are overwritten from the
   dump: `identifier`, `name`, `tier`, `item_power` and the
   category/subcategory mapping come from the game data;
3. `consumable_craftings` is regenerated (base recipe + one per enchantment,
   with `per_craft` from `@amountcrafted` and requirements resolved to
   identifiers/icons);
4. a report is printed and written to `data/backups/<timestamp>/report.json`:
   added rows, renames, corrected identifiers, item-power changes, items the
   dump lacks (kept), unmapped subcategories and skipped cosmetics.

Preserved on overwrite: the row `id` of every matched item (so
`*_stats` and item↔spell pivots stay attached), the wiki `path`, and the
consumable `info` text.

Category tree: `categories.json` is curated by hand and never rewritten by the
import, but it was extended for the families only the game data exposes —
`Gathering Armor` (Garb / Cap / Workboots), `Avalonian Capes`,
`Smuggler Capes`, `Backpacks`, the newer potion types (Acid, Berserk,
Cleansing, Gathering, Hellfire, Lifeward, Calming, Tornado) and `Fish`. The
importer maps every item onto that tree, so no item is left without a
category; unknown families are reported instead of being invented.

Cosmetics: promotional/vanity content (founder, starter-pack, recruiter,
event, firework, decorative, Game-Master and test entries) is **excluded** —
and removed from the dataset if an earlier import added it (the backup keeps
it). `--include-cosmetics` opts back in.

**Not provided by the dump** (documented limitation, reported per run):
per-quality stats, spells/slot links, and wiki paths for new items. New items
therefore arrive with `path: null` and no stats/spells until a permitted
source for those exists; items whose wiki subcategory is unknown are written
with `subcategory_id: null` rather than inventing a category.

Flags: `--dry-run`, `--refresh` (re-download the dump), `--from-dir DIR` (use a
local checkout instead of downloading), `--include-cosmetics` (also import
founder/vanity/starter-pack items, skipped by default), `--only items|crafting`,
`status` (compare the dataset with the dump without writing).
Downloads are cached in `data/.cache/ao-bin-dumps/` (both gitignored).

Latest import (dump of 2026-11-19): weapons 694 → **802**, armors 426 → **534**,
accessories 174 → **219**, consumables 72 → **100**, crafting rows 238 → **390**;
7 accessories + 1 consumable are wiki-only entries the dump lacks and were kept,
and 45 cosmetics from the earlier import were pruned. Every item now has a
category and subcategory (0 nulls).

## Tests

```bash
cargo test    # 26 tests: unit + router-level integration
cargo clippy  # warning-free
```

## Parity verification against the Laravel app

`scripts/parity_check.sh` runs the same ~70 requests against both apps and
diffs the normalized JSON. Run it against a **copy** of the SQLite DB so
Laravel's cache/log writes never touch the repository's DB file:

```bash
# 1. Temp DB copy + parity token (once)
cp ../database/database.sqlite /tmp/openalbion-parity.sqlite
sqlite3 /tmp/openalbion-parity.sqlite "INSERT INTO api_tokens
  (user_id, name, token, created_at, updated_at)
  VALUES (1, 'parity', 'parity-token', datetime('now'), datetime('now'));"

# 2. Laravel on the copy (from the repo root):
cd .. && cd public && DB_DATABASE=/tmp/openalbion-parity.sqlite \
  WEAPONRY_KEY=paritykey123 php -S 127.0.0.1:8000 \
  ../vendor/laravel/framework/src/Illuminate/Foundation/resources/server.php

# 3. Rust app (the `WEAPONRY_KEY` there is for the *Laravel* side only; the
#    Rust app is public and ignores the header):
cd ../rust && DATA_DIR=./data \
  RATE_LIMIT_PER_MINUTE=1000000 PORT=9911 cargo run

# 4. Compare
LARAVEL_DB=/tmp/openalbion-parity.sqlite bash scripts/parity_check.sh
```

Latest run: **69 passed, 0 failed**, plus one documented skip — Laravel's
`/api/weaponryV2/search` cannot answer locally because it requires Algolia
credentials (the Rust app replaced that endpoint with an in-memory search, so
responses only differ in result *ordering*). The `POST .../bug-report` 422
validation bodies were also verified byte-for-byte against Laravel.

The three web pages are compared by **status code only** — the Rust UI
(home, terms, privacy) is an intentional redesign, so its markup and text no
longer mirror the Laravel blade views.

The AOD price proxy compares response *shape* only (it proxies live market
data that changes between requests; the upstream is also flaky under the 3s
timeout, which either app can hit on a given run).

## Docker

```bash
docker build -t openalbion-api .
docker run -p 8080:8080 openalbion-api
```

The data files are baked into the image; mount a volume over `/app/data` to
use a fresher export.

## Cloudflare Workers

`worker/` runs the same axum router on Cloudflare Workers, compiled to
WebAssembly with [workers-rs](https://github.com/cloudflare/workers-rs). The
`data/*.json` tables are embedded in the wasm (~0.8 MB gzipped) and parsed
once per isolate on the first request.

One-time setup (needs a rustup-managed toolchain; Homebrew's `rust` has no
wasm target):

```bash
brew install rustup && export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
cargo install worker-build
```

Run locally / deploy:

```bash
cd worker
npx wrangler dev        # http://localhost:8787
npx wrangler login      # once
npx wrangler deploy
```

Optional Slack forwarding of bug reports: `npx wrangler secret put SLACK_WEBHOOK_URL`.

Differences from the native server on Workers:

- Rate limiting (60/min per IP on `/api/*`) uses the Workers Rate Limiting
  binding (`API_RATE_LIMITER` in `wrangler.toml`), keyed by `CF-Connecting-IP`.
- Bug reports are written to the Workers log (`wrangler tail` / dashboard)
  instead of `bug-reports.jsonl`; the Slack webhook is awaited inline.
- The AOD price cache is per isolate.
- Cold start parses the dataset (~100+ ms CPU). That exceeds the Workers Free
  plan's 10 ms CPU limit on the first request per isolate, so use the Paid plan
  for production.

## Differences from the Laravel app (all deliberate)

1. v1/v2 token check removed — all public API versions are now public.
2. `/api/weaponry{,V2}` are public — Laravel still requires the
   `X-WEAPONRY-KEY` header there.
3. No database: data comes from JSON files; bug reports go to
   `bug-reports.jsonl` (+ optional Slack webhook).
4. No HTTP-request log table — structured `tracing` logs instead.
5. `weaponryV2` search uses an in-memory substring search instead of
   Algolia (ordering differs; no external service needed).
6. AOD price proxy cache TTL is 180s (the PHP comment's stated intent)
   instead of the 3600s the config actually used.
7. Upstream AOD failures answer `502` JSON instead of Laravel's 500 HTML
   page; unknown routes answer JSON instead of Laravel's HTML error page.
8. Non-numeric `{weaponId}`-style path parameters answer `{"data": []}`
   instead of Laravel's SQL error page.

## License

MIT, like the original project.
