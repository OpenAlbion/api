# OpenAlbion

OpenAlbion is a free and open-source platform providing **Albion Online data and an
API**, plus the [Weaponry](https://play.google.com/store/apps/details?id=com.openalbion.weaponry)
companion app.

The project is now a **Rust API** (no database — the dataset is plain JSON) that runs as a
native binary or on Cloudflare Workers. The original Laravel implementation that produced
the dataset is kept for reference in [`Laravel (legacy)/`](<Laravel (legacy)/README.md>).

- Website / docs: <https://openalbion.com>
- Live API: <https://openalbion.pyaesoneaungrgn.workers.dev>
- License: MIT

## Repository layout

```
rust/                    The API (Rust + axum), the dataset, and the data importer
  src/                   Handlers, DTOs, state, web pages
  src/importer/          ao-bin-dumps importer (feature `import`)
  data/                  JSON dataset the API serves
  web/                   Home / terms / privacy pages (embedded in the binary)
  worker/                Cloudflare Workers build (same router, compiled to wasm)
  scripts/               Data export + Laravel parity checker
Laravel (legacy)/        The original Laravel app: DB migrations, models, console
                         commands and the former API implementation
README.md                This file
```

## The API

All endpoints are public — no API key, no token, no sign-up. Rate limit: 60 requests
per minute per IP.

| Prefix | Contents |
|---|---|
| `/api/v3` | Current public API: categories, weapons, armors, accessories, consumables, spells, stats, crafting |
| `/api/v1`, `/api/v2` | Older response shapes, kept for existing clients |
| `/api/weaponry`, `/api/weaponryV2` | Endpoints used by the Weaponry app (market prices, search, bug report, version check) |
| `/`, `/terms-of-service`, `/privacy-policy` | Website pages |

Examples:

```bash
curl https://openalbion.pyaesoneaungrgn.workers.dev/api/v3/weapons?tier=8
curl https://openalbion.pyaesoneaungrgn.workers.dev/api/v3/spells/weapon/1
curl https://openalbion.pyaesoneaungrgn.workers.dev/api/v3/consumable-craftings/consumable/1
```

Full endpoint documentation: <https://openalbion.com>

## Running locally

Requirements: Rust (stable). The dataset is committed, so no external service is needed.

```bash
cd rust
cargo run --release            # http://0.0.0.0:8080
```

Configuration (environment or `rust/.env`): `HOST`, `PORT`, `DATA_DIR`,
`SLACK_WEBHOOK_URL`, `RATE_LIMIT_PER_MINUTE`, `AOD_CACHE_SECONDS`, `RUST_LOG`.
See [`rust/README.md`](rust/README.md) for the full reference.

Tests:

```bash
cd rust
cargo test        # 52 tests: unit + router-level integration
cargo clippy      # warning-free
```

## Deploying

**Cloudflare Workers** (the live deployment):

```bash
cd rust/worker
npx wrangler deploy
```

**Docker** (native binary): see [`rust/Dockerfile`](rust/Dockerfile).

## Dataset

The dataset is 14 JSON tables (`rust/data/`) loaded into memory at startup — no database
server is involved:

```
categories  weapons  armors  accessories  consumables  spells
weapon_spells  armor_spells  accessory_spells
weapon_stats  armor_stats  accessory_stats  consumable_stats
consumable_craftings
```

Current contents: 802 weapons, 534 armors, 219 accessories, 100 consumables,
460 spells, ~29k stat rows and 390 crafting recipes.

Item data is refreshed from the public game-data dump
[ao-bin-dumps](https://github.com/ao-data/ao-bin-dumps) with the `openalbion-import`
binary:

```bash
cd rust
cargo run --release --features import --bin openalbion-import -- --dry-run   # preview
cargo run --release --features import --bin openalbion-import               # apply
```

Every run backs the previous dataset up to `rust/data/backups/<timestamp>/` first, so
changes are always reversible. Details, guarantees and limitations are documented in
[`rust/README.md`](rust/README.md).

> The Albion Online Wiki's bot protection is respected: **no crawler or Cloudflare bypass
> ships with this project**. Item data comes from the game-data dump; per-quality stats,
> spells and wiki links for newly added items are not available from it.

## Legacy Laravel application

[`Laravel (legacy)/`](<Laravel (legacy)/README.md>) holds the original PHP/Laravel
implementation: the Eloquent models and migrations that defined the dataset schema, the
artisan commands that scraped the wiki, and the former API controllers. It is kept for
historical reference and is no longer the deployed service — the Rust API replaces it.
Its database (`Laravel (legacy)/database/database.sqlite`) is the snapshot the JSON
dataset was exported from.

## Credits

Built by [Pyae Sone Aung](https://github.com/pyaesoneaung) and contributors.
Data collected from the Albion Online Wiki and the game's own data dumps.
Not affiliated with Sandbox Interactive.
