# Versions & extras

The endpoint pages document the current **v3** API. This page covers the versioned response
shapes, error behaviour and the Weaponry endpoints that the companion app uses.

## API versions

| Version | Base path | Notes |
| :------------- | :------------- | :------------- |
| v3 | `/api/v3` | Current API. Item resources nest `category` and `subcategory`. Includes consumables. |
| v2 | `/api/v2` | Older shape: flat item resources. Includes consumables. Public — no token. |
| v1 | `/api/v1` | Oldest shape: flat item resources. Does not include consumables. Public — no token. |

Every version supports the same filters and path parameters. The differences are in the
resource shape and in which categories each version exposes:

- v3 item objects include a nested `category` and `subcategory` object.
- v1 and v2 item objects are flat: they omit the nested `category` and `subcategory` objects.
- Category lists are filtered by version (v1 returns the oldest set, v2/v3 the larger set).
- Consumables and consumable stats/craftings exist from v2 onwards.

## Rate limits and errors

- **60 requests per minute per IP** across `/api/*`.
- Over the limit: `429 Too Many Attempts` with `Retry-After` and `x-ratelimit-limit` headers and a `{"message":"Too Many Attempts."}` body.
- Unknown API routes return a JSON `404`: `{"message":"Not Found"}`.
- CORS is enabled for all origins, so browser apps can call the API directly.

## Search

A search endpoint is available for the Weaponry app:

- `GET /api/weaponry/search?search=battleaxe` — up to three weapons, armors and accessories, in that order.

Search is a case-insensitive substring match on the item name. Each result carries `type`,
`type_id`, `name`, `tier`, `item_power`, `identifier`, `info` and `icon`.

```http
GET https://api.openalbion.com/api/weaponry/search?search=battleaxe
```

## Market prices

`GET /api/weaponry/aod/{region}/item/{item_id}/price` proxies live prices from Albion Online
Data. `region` is the server region host prefix: `europe`, `americas` or `asia`.

Query parameters are forwarded to Albion Online Data as-is, so `locations`, `qualities` and the
other AOD options work:

```http
GET https://api.openalbion.com/api/weaponry/aod/europe/item/T4_BAG/price?locations=Caerleon&qualities=1
```

Responses are cached for 180 seconds. If the upstream service cannot be reached the API answers
`502` with a JSON message.

## Version check

`GET /api/weaponry/version-check` returns the latest Weaponry app version:

```json
{
  "title": "Update Available!",
  "description": "[+] Add Food & Potions",
  "version": "1.0.7",
  "force": false
}
```

## Bug reports

`POST /api/weaponry/bug-report` accepts a JSON body with three required fields:

- `category` — string, at most 50 characters.
- `description` — string, at most 250 characters.
- `debug` — array of debugging data.

```json
{
  "category": "UI",
  "description": "The price panel is empty",
  "debug": [{ "platform": "android", "version": "1.0.7" }]
}
```

A valid report answers `{"message":"success"}`. Invalid bodies answer `422` with a validation
error payload.

## Notes on the examples

The JSON on the endpoint pages is illustrative. Long arrays are shortened with `{...}`, and the
numbers change as the game and its data dumps are updated. Use the endpoint pages for the field
names and query parameters, and the [playground](/#playground) to inspect the live response.
