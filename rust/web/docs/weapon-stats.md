# Weapon Stats

## Get Weapon Stats By Weapon

Retrieve information about weapon stats by weapon.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/weapon-stats/weapon/{weapon_id}**

Example Request:
```http
GET https://api.openalbion.com/api/v3/weapon-stats/weapon/2
```

Example Response:
```json
{
    "data": [
        {
            "enchantment": 0,
            "icon": "https://render.albiononline.com/v1/item/T4_MAIN_AXE@0.png?quality=0&size=217&locale=en",
            "stats": [
                {
                    "id": 6,
                    "quality": "Normal",
                    "enchantment": 0,
                    "weapon": {
                        "id": 2,
                        "name": "Adept's Battleaxe",
                        "tier": "4.0",
                        "item_power": 700,
                        "identifier": "T4_MAIN_AXE",
                        "icon": "https://render.albiononline.com/v1/item/T4_MAIN_AXE@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "700"
                        },
                        {
                            "name": "Physical damage",
                            "value": "71"
                        },
                        {
                            "name": "Resilience penetration",
                            "value": "50%"
                        },
                        {
                            "name": "Max Hit Points",
                            "value": "75"
                        },
                        {
                            "name": "Hit Points Regeneration(Bonus)",
                            "value": "5/s"
                        },
                        {
                            "name": "DPS",
                            "value": "79/s"
                        }
                    ]
                },
                {
                    "id": 7,
                    "quality": "Good",
                    "enchantment": 0,
                    "weapon": {
                        "id": 2,
                        "name": "Adept's Battleaxe",
                        "tier": "4.0",
                        "item_power": 700,
                        "identifier": "T4_MAIN_AXE",
                        "icon": "https://render.albiononline.com/v1/item/T4_MAIN_AXE@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "720"
                        },
                        {
                            "name": "Physical damage",
                            "value": "73"
                        },
                        {
                            "name": "Resilience penetration",
                            "value": "50%"
                        },
                        {
                            "name": "Max Hit Points",
                            "value": "76"
                        },
                        {
                            "name": "Hit Points Regeneration(Bonus)",
                            "value": "5/s"
                        },
                        {
                            "name": "DPS",
                            "value": "80/s"
                        }
                    ]
                },
                {
                    "id": 8,
                    "quality": "Outstanding",
                    "enchantment": 0,
                    "weapon": {
                        "id": 2,
                        "name": "Adept's Battleaxe",
                        "tier": "4.0",
                        "item_power": 700,
                        "identifier": "T4_MAIN_AXE",
                        "icon": "https://render.albiononline.com/v1/item/T4_MAIN_AXE@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "740"
                        },
                        {
                            "name": "Physical damage",
                            "value": "74"
                        },
                        {
                            "name": "Resilience penetration",
                            "value": "50%"
                        },
                        {
                            "name": "Max Hit Points",
                            "value": "77"
                        },
                        {
                            "name": "Hit Points Regeneration(Bonus)",
                            "value": "5/s"
                        },
                        {
                            "name": "DPS",
                            "value": "81/s"
                        }
                    ]
                },
                {
                    "id": 9,
                    "quality": "Excellent",
                    "enchantment": 0,
                    "weapon": {
                        "id": 2,
                        "name": "Adept's Battleaxe",
                        "tier": "4.0",
                        "item_power": 700,
                        "identifier": "T4_MAIN_AXE",
                        "icon": "https://render.albiononline.com/v1/item/T4_MAIN_AXE@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "760"
                        },
                        {
                            "name": "Physical damage",
                            "value": "75"
                        },
                        {
                            "name": "Resilience penetration",
                            "value": "50%"
                        },
                        {
                            "name": "Max Hit Points",
                            "value": "78"
                        },
                        {
                            "name": "Hit Points Regeneration(Bonus)",
                            "value": "5/s"
                        },
                        {
                            "name": "DPS",
                            "value": "82/s"
                        }
                    ]
                },
                {
                    "id": 10,
                    "quality": "Masterpiece",
                    "enchantment": 0,
                    "weapon": {
                        "id": 2,
                        "name": "Adept's Battleaxe",
                        "tier": "4.0",
                        "item_power": 700,
                        "identifier": "T4_MAIN_AXE",
                        "icon": "https://render.albiononline.com/v1/item/T4_MAIN_AXE@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "800"
                        },
                        {
                            "name": "Physical damage",
                            "value": "77"
                        },
                        {
                            "name": "Resilience penetration",
                            "value": "50%"
                        },
                        {
                            "name": "Max Hit Points",
                            "value": "80"
                        },
                        {
                            "name": "Hit Points Regeneration(Bonus)",
                            "value": "5/s"
                        },
                        {
                            "name": "DPS",
                            "value": "85/s"
                        }
                    ]
                }
            ]
        },
        {...}
    ]
}
```