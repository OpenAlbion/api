# Accessories

## Get All Accessories

Retrieve information about accessories.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/accessories**

Example Request:
```http
GET https://api.openalbion.com/api/v3/accessories
```

Example Response:
```json
{
    "data": [
        {
            "id": 1,
            "name": "Novice's Cape",
            "tier": "2.0",
            "item_power": 500,
            "identifier": "T2_CAPE",
            "icon": "https://render.albiononline.com/v1/item/T2_CAPE@0.png?quality=0&size=217&locale=en",
            "category": { "id": 35, "name": "Cape", "type": "accessory" },
            "subcategory": { "id": 37, "name": "Normal Capes", "type": "accessory" }
        },
        {
            "id": 2,
            "name": "Journeyman's Cape",
            "tier": "3.0",
            "item_power": 600,
            "identifier": "T3_CAPE",
            "icon": "https://render.albiononline.com/v1/item/T3_CAPE@0.png?quality=0&size=217&locale=en",
            "category": { "id": 35, "name": "Cape", "type": "accessory" },
            "subcategory": { "id": 37, "name": "Normal Capes", "type": "accessory" }
        },
        {...}
    ]
}
```

## Get Accessories By Category

Retrieve information about accessories by category.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/accessories**
- Query Parameters:
  - `category_id`: Filter the accessories by category. Example: `35`

Example Request:
```http
GET https://api.openalbion.com/api/v3/accessories?category_id=35
```

Example Response:
```json
{
    "data": [
        {
            "id": 1,
            "name": "Novice's Cape",
            "tier": "2.0",
            "item_power": 500,
            "identifier": "T2_CAPE",
            "icon": "https://render.albiononline.com/v1/item/T2_CAPE@0.png?quality=0&size=217&locale=en",
            "category": { "id": 35, "name": "Cape", "type": "accessory" },
            "subcategory": { "id": 37, "name": "Normal Capes", "type": "accessory" }
        },
        {
            "id": 2,
            "name": "Journeyman's Cape",
            "tier": "3.0",
            "item_power": 600,
            "identifier": "T3_CAPE",
            "icon": "https://render.albiononline.com/v1/item/T3_CAPE@0.png?quality=0&size=217&locale=en",
            "category": { "id": 35, "name": "Cape", "type": "accessory" },
            "subcategory": { "id": 37, "name": "Normal Capes", "type": "accessory" }
        },
        {...}
    ]
}
```

## Get Accessories By Subcategory

Retrieve information about accessories by subcategory.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/accessories**
- Query Parameters:
  - `subcategory_id`: Filter the accessories by subcategory. Example: `37`

Example Request:
```http
GET https://api.openalbion.com/api/v3/accessories?subcategory_id=37
```

Example Response:
```json
{
    "data": [
        {
            "id": 1,
            "name": "Novice's Cape",
            "tier": "2.0",
            "item_power": 500,
            "identifier": "T2_CAPE",
            "icon": "https://render.albiononline.com/v1/item/T2_CAPE@0.png?quality=0&size=217&locale=en",
            "category": { "id": 35, "name": "Cape", "type": "accessory" },
            "subcategory": { "id": 37, "name": "Normal Capes", "type": "accessory" }
        },
        {
            "id": 2,
            "name": "Journeyman's Cape",
            "tier": "3.0",
            "item_power": 600,
            "identifier": "T3_CAPE",
            "icon": "https://render.albiononline.com/v1/item/T3_CAPE@0.png?quality=0&size=217&locale=en",
            "category": { "id": 35, "name": "Cape", "type": "accessory" },
            "subcategory": { "id": 37, "name": "Normal Capes", "type": "accessory" }
        },
        {...}
    ]
}
```

## Get Accessories By Tier

Retrieve information about accessories by tier.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/accessories**
- Query Parameters:
  - `tier`: Filter the accessories by tier. Example: `4`

Example Request:
```http
GET https://api.openalbion.com/api/v3/accessories?tier=4
```

Example Response:
```json
{
    "data": [
        {
            "id": 3,
            "name": "Adept's Bridgewatch Cape",
            "tier": "4.0",
            "item_power": 700,
            "identifier": "T4_CAPEITEM_FW_BRIDGEWATCH",
            "icon": "https://render.albiononline.com/v1/item/T4_CAPEITEM_FW_BRIDGEWATCH@0.png?quality=0&size=217&locale=en",
            "category": { "id": 35, "name": "Cape", "type": "accessory" },
            "subcategory": { "id": 38, "name": "Faction Capes", "type": "accessory" }
        },
        {
            "id": 4,
            "name": "Adept's Caerleon Cape",
            "tier": "4.0",
            "item_power": 700,
            "identifier": "T4_CAPEITEM_FW_CAERLEON",
            "icon": "https://render.albiononline.com/v1/item/T4_CAPEITEM_FW_CAERLEON@0.png?quality=0&size=217&locale=en",
            "category": { "id": 35, "name": "Cape", "type": "accessory" },
            "subcategory": { "id": 38, "name": "Faction Capes", "type": "accessory" }
        },
        {...}
    ]
}
```