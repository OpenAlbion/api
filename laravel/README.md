# OpenAlbion — Laravel (legacy implementation)

This is the **original Laravel implementation** of the OpenAlbion API and dataset
pipeline. It is kept for reference only: the deployed service is now the Rust API in
[`../rust/`](../rust/README.md), and this folder is no longer built or deployed.

## Why it is still here

- **Schema source of truth.** The Eloquent models and migrations in `database/migrations/`
  define the 14 tables that the Rust API serves as JSON
  (`categories`, `weapons`, `armors`, `accessories`, `consumables`, `spells`, the three
  spell pivots, the four stat tables and `consumable_craftings`).
- **Dataset origin.** `database/database.sqlite` is the snapshot the JSON dataset was
  exported from (`../rust/scripts/export_sqlite.py`). Its root-mapped JSON columns,
  nullable fields and id allocation are what the Rust importer preserves.
- **Reference implementation.** `app/Http/Controllers/Api/V1..V3` plus
  `app/Http/Resources/` document the exact response shapes each API version returns, and
  `app/Console/Commands/` documents how the wiki data was collected originally.

It also contains functionality the Rust rewrite deliberately dropped: user accounts,
Google sign-in, API-token management, the dashboard/profile pages and request logging.

## Running it (optional, historical)

Requires PHP 8.3+, Composer and Node.

```bash
cd laravel
composer install
cp .env.example .env
php artisan key:generate
php artisan migrate        # SQLite database already present in database/
php artisan serve          # http://127.0.0.1:8000
```

The front end is built with Vite/Tailwind:

```bash
npm install
npm run build
```

## What it contains

| Path | Purpose |
|---|---|
| `app/Models`, `database/migrations` | The dataset schema (the JSON tables mirror these) |
| `app/Http/Controllers/Api/{V1,V2,V3}` | The former public API |
| `app/Http/Controllers/Api/Weaponry` | Endpoints used by the mobile app |
| `app/Http/Resources` | The response shapes per API version |
| `app/Console/Commands` | The original wiki fetch/import commands |
| `app/Services/{DomCrawler,Wiki}` | The wiki scraping layer (Browsershot + DomCrawler) |
| `routes/` | Route definitions for all API versions and web pages |
| `database/database.sqlite` | The data snapshot the JSON dataset came from |

## Status

Frozen. No further changes are expected here; new functionality goes into the Rust API.
The wiki-scraping commands are not supported any more — the wiki's bot protection is
respected and item data is refreshed from the game-data dump instead (see
[`../rust/README.md`](../rust/README.md)).
