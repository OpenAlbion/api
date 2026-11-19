# Introduction

Welcome to the OpenAlbion API — free, open-source Albion Online data for your website, app or
tool. The documentation that used to live at `openalbion.com` now lives here, right next to the
API it describes.

## Base URL

Every endpoint lives under:

```text
https://api.openalbion.com/api
```

The current public API is **v3**. The older `v1` and `v2` response shapes are still served for
existing clients; see [Versions & extras](/docs/versions) for the differences.

## No key, no sign-up

Every public endpoint is open to everyone:

- No API key, token or account.
- CORS is enabled, so browser apps can call the API directly.
- Fair-use limit: **60 requests per minute per IP**. Over the limit you get `429 Too Many Attempts` with a `Retry-After` header.
- Responses are JSON. List endpoints return a `data` array; errors return a `message` string.
- The dataset is plain JSON loaded into a Rust API — there is no database to time out.

## Response envelope

List and detail endpoints wrap their payload in `data`:

```json
{
  "data": [
    {
      "id": 1,
      "name": "Journeyman's Battleaxe",
      "tier": "3.0"
    }
  ]
}
```

Errors use a short, stable shape:

```json
{
  "message": "Not Found"
}
```

## Quick start

Fetch every Tier 8 weapon:

```bash
curl "https://api.openalbion.com/api/v3/weapons?tier=8"
```

Or from JavaScript:

```js
const response = await fetch("https://api.openalbion.com/api/v3/weapons?tier=8");
const { data } = await response.json();
console.log(data.length);
```

## What's inside

- Weapons, armors, accessories and consumables with tier, item power, identifier and icon.
- Categories and subcategories for every item type.
- Spells for weapons, armors and accessories, grouped by slot.
- Per-quality stats for every enchantment level.
- Crafting recipes with ingredients and per-craft output.
- Search, version checks and a live market-price proxy through the Weaponry endpoints.

## Data and attribution

Item data is collected from the Albion Online Wiki and the game's public data dumps. OpenAlbion
is a community project, is MIT licensed, and is **not affiliated with Sandbox Interactive**.

Ready? Browse the endpoints in the sidebar, or head back to the [home page](/) and try a request
in the live playground.
