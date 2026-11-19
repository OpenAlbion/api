# Consumable Stats

## Get Consumable Stats By Consumable

Retrieve information about consumable stats by consumable.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/consumable-stats/consumable/{consumable_id}**

Example Request:
```http
GET https://api.openalbion.com/api/v3/consumable-stats/consumable/18
```

Example Response:
```json
{
    "data": [
        {
            "enchantment": 0,
            "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@0.png?quality=0&size=217&locale=en",
            "stats": [
                {
                    "id": 69,
                    "quality": "Normal",
                    "enchantment": 0,
                    "consumable": {
                        "id": 18,
                        "name": "Midwater Octopus Salad",
                        "tier": "4.0",
                        "item_power": 700,
                        "identifier": "T4_MEAL_SALAD_FISH",
                        "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@0.png?quality=0&size=217&locale=en",
                        "info": "Players may craft Midwater Octopus Salad at the Cook\nMidwater Octopus Salad may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Midwater Octopus Salad last for 2 minutes\nMidwater Octopus Salad per craft: 1\n"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "700"
                        },
                        {
                            "name": "Nutrition",
                            "value": "380"
                        },
                        {
                            "name": "Weight",
                            "value": "4.8 kg"
                        },
                        {
                            "name": "Crafting Speed",
                            "value": "+99"
                        },
                        {
                            "name": "Crafting Quality",
                            "value": "+99"
                        }
                    ]
                }
            ]
        },
        {
            "enchantment": 1,
            "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@1.png?quality=0&size=217&locale=en",
            "stats": [
                {
                    "id": 70,
                    "quality": "Normal",
                    "enchantment": 1,
                    "consumable": {
                        "id": 18,
                        "name": "Midwater Octopus Salad",
                        "tier": "4.0",
                        "item_power": 700,
                        "identifier": "T4_MEAL_SALAD_FISH",
                        "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@0.png?quality=0&size=217&locale=en",
                        "info": "Players may craft Midwater Octopus Salad at the Cook\nMidwater Octopus Salad may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Midwater Octopus Salad last for 2 minutes\nMidwater Octopus Salad per craft: 1\n"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "500"
                        },
                        {
                            "name": "Nutrition",
                            "value": "380"
                        },
                        {
                            "name": "Weight",
                            "value": "4.8 kg"
                        },
                        {
                            "name": "Crafting Speed",
                            "value": "+123"
                        },
                        {
                            "name": "Crafting Quality",
                            "value": "+123"
                        }
                    ]
                }
            ]
        },
        {
            "enchantment": 2,
            "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@2.png?quality=0&size=217&locale=en",
            "stats": [
                {
                    "id": 71,
                    "quality": "Normal",
                    "enchantment": 2,
                    "consumable": {
                        "id": 18,
                        "name": "Midwater Octopus Salad",
                        "tier": "4.0",
                        "item_power": 700,
                        "identifier": "T4_MEAL_SALAD_FISH",
                        "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@0.png?quality=0&size=217&locale=en",
                        "info": "Players may craft Midwater Octopus Salad at the Cook\nMidwater Octopus Salad may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Midwater Octopus Salad last for 2 minutes\nMidwater Octopus Salad per craft: 1\n"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "700"
                        },
                        {
                            "name": "Nutrition",
                            "value": "380"
                        },
                        {
                            "name": "Weight",
                            "value": "4.8 kg"
                        },
                        {
                            "name": "Crafting Speed",
                            "value": "+148"
                        },
                        {
                            "name": "Crafting Quality",
                            "value": "+148"
                        }
                    ]
                }
            ]
        },
        {
            "enchantment": 3,
            "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@3.png?quality=0&size=217&locale=en",
            "stats": [
                {
                    "id": 72,
                    "quality": "Normal",
                    "enchantment": 3,
                    "consumable": {
                        "id": 18,
                        "name": "Midwater Octopus Salad",
                        "tier": "4.0",
                        "item_power": 700,
                        "identifier": "T4_MEAL_SALAD_FISH",
                        "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@0.png?quality=0&size=217&locale=en",
                        "info": "Players may craft Midwater Octopus Salad at the Cook\nMidwater Octopus Salad may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Midwater Octopus Salad last for 2 minutes\nMidwater Octopus Salad per craft: 1\n"
                    },
                    "stats": [
                        {
                            "name": "Item Power",
                            "value": "800"
                        },
                        {
                            "name": "Nutrition",
                            "value": "380"
                        },
                        {
                            "name": "Weight",
                            "value": "4.8 kg"
                        },
                        {
                            "name": "Crafting Speed",
                            "value": "+173"
                        },
                        {
                            "name": "Crafting Quality",
                            "value": "+173"
                        }
                    ]
                }
            ]
        }
    ]
}
```