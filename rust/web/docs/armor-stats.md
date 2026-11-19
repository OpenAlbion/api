# Armor Stats

## Get Armor Stats By Armor

Retrieve information about armor stats by armor.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/armor-stats/armor/{armor_id}**

Example Request:
```http
GET https://api.openalbion.com/api/v3/armor-stats/armor/1
```

Example Response:
```json
{
    "data": [
        {
            "enchantment": 0,
            "icon": "https://render.albiononline.com/v1/item/T2_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en",
            "stats": [
                {
                    "id": 1,
                    "quality": "Normal",
                    "enchantment": 0,
                    "armor": {
                        "id": 1,
                        "name": "Novice's Scholar Robe",
                        "tier": "2.0",
                        "item_power": 500,
                        "identifier": "T2_ARMOR_CLOTH_SET1",
                        "icon": "https://render.albiononline.com/v1/item/T2_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "300"
                        },
                        {
                            "name": "Armor",
                            "value": "84"
                        },
                        {
                            "name": "Magical Resistance",
                            "value": "104"
                        },
                        {
                            "name": "Max Hit Points",
                            "value": "115"
                        },
                        {...}
                    ]
                },
                {
                    "id": 2,
                    "quality": "Good",
                    "enchantment": 0,
                    "armor": {
                        "id": 1,
                        "name": "Novice's Scholar Robe",
                        "tier": "2.0",
                        "item_power": 500,
                        "identifier": "T2_ARMOR_CLOTH_SET1",
                        "icon": "https://render.albiononline.com/v1/item/T2_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "320"
                        },
                        {
                            "name": "Armor",
                            "value": "85"
                        },
                        {
                            "name": "Magical Resistance",
                            "value": "106"
                        },
                        {
                            "name": "Max Hit Points",
                            "value": "123"
                        },
                        {...}
                    ]
                },
                {
                    "id": 3,
                    "quality": "Outstanding",
                    "enchantment": 0,
                    "armor": {
                        "id": 1,
                        "name": "Novice's Scholar Robe",
                        "tier": "2.0",
                        "item_power": 500,
                        "identifier": "T2_ARMOR_CLOTH_SET1",
                        "icon": "https://render.albiononline.com/v1/item/T2_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "340"
                        },
                        {
                            "name": "Armor",
                            "value": "86"
                        },
                        {
                            "name": "Magical Resistance",
                            "value": "107"
                        },
                        {
                            "name": "Max Hit Points",
                            "value": "131"
                        },
                        {...}
                    ]
                },
                {
                    "id": 4,
                    "quality": "Excellent",
                    "enchantment": 0,
                    "armor": {
                        "id": 1,
                        "name": "Novice's Scholar Robe",
                        "tier": "2.0",
                        "item_power": 500,
                        "identifier": "T2_ARMOR_CLOTH_SET1",
                        "icon": "https://render.albiononline.com/v1/item/T2_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "360"
                        },
                        {
                            "name": "Armor",
                            "value": "87"
                        },
                        {
                            "name": "Magical Resistance",
                            "value": "108"
                        },
                        {
                            "name": "Max Hit Points",
                            "value": "140"
                        },
                        {...}
                    ]
                },
                {
                    "id": 5,
                    "quality": "Masterpiece",
                    "enchantment": 0,
                    "armor": {
                        "id": 1,
                        "name": "Novice's Scholar Robe",
                        "tier": "2.0",
                        "item_power": 500,
                        "identifier": "T2_ARMOR_CLOTH_SET1",
                        "icon": "https://render.albiononline.com/v1/item/T2_ARMOR_CLOTH_SET1@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "400"
                        },
                        {
                            "name": "Armor",
                            "value": "89"
                        },
                        {
                            "name": "Magical Resistance",
                            "value": "110"
                        },
                        {
                            "name": "Max Hit Points",
                            "value": "157"
                        },
                        {...}
                    ]
                }
            ]
        }
    ]
}
```