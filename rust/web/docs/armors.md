# Armors

## Get All Armors

Retrieve information about armors.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/armors**

Example Request:
```http
GET https://api.openalbion.com/api/v3/armors
```

Example Response:
```json
{
    "data": [
        {
            "id": 1,
            "name": "Novice's Scholar Robe",
            "tier": "2.0",
            "item_power": 500,
            "identifier": "T2_ARMOR_CLOTH_SET1",
            "icon": "https://render.albiononline.com/v1/item/T2_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en",
            "category": { "id": 23, "name": "Cloth Armor", "type": "armor" },
            "subcategory": { "id": 24, "name": "Robes", "type": "armor" }
        },
        {
            "id": 2,
            "name": "Journeyman's Scholar Robe",
            "tier": "3.0",
            "item_power": 600,
            "identifier": "T3_ARMOR_CLOTH_SET1",
            "icon": "https://render.albiononline.com/v1/item/T3_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en",
            "category": { "id": 23, "name": "Cloth Armor", "type": "armor" },
            "subcategory": { "id": 24, "name": "Robes", "type": "armor" }
        },
        {...}
    ]
}
```

## Get Armors By Category

Retrieve information about armors by category.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/armors**
- Query Parameters:
  - `category_id`: Filter the armors by category. Example: `23`

Example Request:
```http
GET https://api.openalbion.com/api/v3/armors?category_id=23
```

Example Response:
```json
{
    "data": [
        {
            "id": 1,
            "name": "Novice's Scholar Robe",
            "tier": "2.0",
            "item_power": 500,
            "identifier": "T2_ARMOR_CLOTH_SET1",
            "icon": "https://render.albiononline.com/v1/item/T2_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en",
            "category": { "id": 23, "name": "Cloth Armor", "type": "armor" },
            "subcategory": { "id": 24, "name": "Robes", "type": "armor" }
        },
        {
            "id": 2,
            "name": "Journeyman's Scholar Robe",
            "tier": "3.0",
            "item_power": 600,
            "identifier": "T3_ARMOR_CLOTH_SET1",
            "icon": "https://render.albiononline.com/v1/item/T3_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en",
            "category": { "id": 23, "name": "Cloth Armor", "type": "armor" },
            "subcategory": { "id": 24, "name": "Robes", "type": "armor" }
        },
        {...}
    ]
}
```

## Get Armors By Subcategory

Retrieve information about armors by subcategory.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/armors**
- Query Parameters:
  - `subcategory_id`: Filter the armors by subcategory. Example: `24`

Example Request:
```http
GET https://api.openalbion.com/api/v3/armors?subcategory_id=24
```

Example Response:
```json
{
    "data": [
        {
            "id": 1,
            "name": "Novice's Scholar Robe",
            "tier": "2.0",
            "item_power": 500,
            "identifier": "T2_ARMOR_CLOTH_SET1",
            "icon": "https://render.albiononline.com/v1/item/T2_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en",
            "category": { "id": 23, "name": "Cloth Armor", "type": "armor" },
            "subcategory": { "id": 24, "name": "Robes", "type": "armor" }
        },
        {
            "id": 2,
            "name": "Journeyman's Scholar Robe",
            "tier": "3.0",
            "item_power": 600,
            "identifier": "T3_ARMOR_CLOTH_SET1",
            "icon": "https://render.albiononline.com/v1/item/T3_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en",
            "category": { "id": 23, "name": "Cloth Armor", "type": "armor" },
            "subcategory": { "id": 24, "name": "Robes", "type": "armor" }
        },
        {...}
    ]
}
```

## Get Armors By Tier

Retrieve information about armors by tier.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/armors**
- Query Parameters:
  - `tier`: Filter the armors by tier. Example: `4`

Example Request:
```http
GET https://api.openalbion.com/api/v3/armors?tier=4
```

Example Response:
```json
{
    "data": [
        {
            "id": 3,
            "name": "Adept's Cleric Robe",
            "tier": "4.0",
            "item_power": 700,
            "identifier": "T4_ARMOR_CLOTH_SET2",
            "icon": "https://render.albiononline.com/v1/item/T4_ARMOR_CLOTH_SET2@0.png?quality=0&size=217&locale=en",
            "category": { "id": 23, "name": "Cloth Armor", "type": "armor" },
            "subcategory": { "id": 24, "name": "Robes", "type": "armor" }
        },
        {
            "id": 4,
            "name": "Adept's Cultist Robe",
            "tier": "4.0",
            "item_power": 775,
            "identifier": "T4_ARMOR_CLOTH_MORGANA",
            "icon": "https://render.albiononline.com/v1/item/T4_ARMOR_CLOTH_MORGANA@0.png?quality=0&size=217&locale=en",
            "category": { "id": 23, "name": "Cloth Armor", "type": "armor" },
            "subcategory": { "id": 24, "name": "Robes", "type": "armor" }
        },
        {...}
    ]
}
```