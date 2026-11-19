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
