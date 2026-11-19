# Accessory Stats

## Get Accessory Stats By Accessory

Retrieve information about accessory stats by accessory.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/accessory-stats/accessory/{accessory_id}**

Example Request:
```http
GET https://api.openalbion.com/api/v3/accessory-stats/accessory/1
```

Example Response:
```json
{
    "data": [
        {
            "enchantment": 0,
            "icon": "https://render.albiononline.com/v1/item/T2_CAPE@0.png?quality=0&size=217&locale=en",
            "stats": [
                {
                    "id": 1,
                    "quality": "Normal",
                    "enchantment": 0,
                    "accessory": {
                        "id": 1,
                        "name": "Novice's Cape",
                        "tier": "2.0",
                        "item_power": 500,
                        "identifier": "T2_CAPE",
                        "icon": "https://render.albiononline.com/v1/item/T2_CAPE@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "300"
                        },
                        {
                            "name": "Max Energy",
                            "value": "3.6"
                        },
                        {
                            "name": "Energy Regeneration",
                            "value": "0.05/s"
                        }
                    ]
                },
                {
                    "id": 2,
                    "quality": "Good",
                    "enchantment": 0,
                    "accessory": {
                        "id": 1,
                        "name": "Novice's Cape",
                        "tier": "2.0",
                        "item_power": 500,
                        "identifier": "T2_CAPE",
                        "icon": "https://render.albiononline.com/v1/item/T2_CAPE@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "320"
                        },
                        {
                            "name": "Max Energy",
                            "value": "3.9"
                        },
                        {
                            "name": "Energy Regeneration",
                            "value": "0.05/s"
                        }
                    ]
                },
                {
                    "id": 3,
                    "quality": "Outstanding",
                    "enchantment": 0,
                    "accessory": {
                        "id": 1,
                        "name": "Novice's Cape",
                        "tier": "2.0",
                        "item_power": 500,
                        "identifier": "T2_CAPE",
                        "icon": "https://render.albiononline.com/v1/item/T2_CAPE@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "340"
                        },
                        {
                            "name": "Max Energy",
                            "value": "4.2"
                        },
                        {
                            "name": "Energy Regeneration",
                            "value": "0.05/s"
                        }
                    ]
                },
                {
                    "id": 4,
                    "quality": "Excellent",
                    "enchantment": 0,
                    "accessory": {
                        "id": 1,
                        "name": "Novice's Cape",
                        "tier": "2.0",
                        "item_power": 500,
                        "identifier": "T2_CAPE",
                        "icon": "https://render.albiononline.com/v1/item/T2_CAPE@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "360"
                        },
                        {
                            "name": "Max Energy",
                            "value": "4.5"
                        },
                        {
                            "name": "Energy Regeneration",
                            "value": "0.06/s"
                        }
                    ]
                },
                {
                    "id": 5,
                    "quality": "Masterpiece",
                    "enchantment": 0,
                    "accessory": {
                        "id": 1,
                        "name": "Novice's Cape",
                        "tier": "2.0",
                        "item_power": 500,
                        "identifier": "T2_CAPE",
                        "icon": "https://render.albiononline.com/v1/item/T2_CAPE@0.png?quality=0&size=217&locale=en"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "400"
                        },
                        {
                            "name": "Max Energy",
                            "value": "5.1"
                        },
                        {
                            "name": "Energy Regeneration",
                            "value": "0.06/s"
                        }
                    ]
                }
            ]
        }
    ]
}
```