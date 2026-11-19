# Consumable Craftings

## Get Consumable Craftings By Consumable

Retrieve information about consumable craftings by consumable.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/consumable-craftings/consumable/{consumable_id}**

Example Request:
```http
GET https://api.openalbion.com/api/v3/consumable-craftings/consumable/18
```

Example Response:
```json
{
    "data": [
        {
            "enchantment": 0,
            "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@0.png?quality=0&size=217&locale=en",
            "crafting": {
                "id": 69,
                "per_craft": 1,
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
                "requirements": [
                    {
                        "icon": "https://render.albiononline.com/v1/item/T5_FISH_SALTWATER_ALL_RARE@0.png?quality=0&size=217&locale=en",
                        "name": "Midwater Octopus",
                        "value": 1,
                        "identifier": "T5_FISH_SALTWATER_ALL_RARE"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_TURNIP@0.png?quality=0&size=217&locale=en",
                        "name": "Turnips",
                        "value": 2,
                        "identifier": "T4_TURNIP"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_BURDOCK@0.png?quality=0&size=217&locale=en",
                        "name": "Crenellated Burdock",
                        "value": 2,
                        "identifier": "T4_BURDOCK"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_MEAT@0.png?quality=0&size=217&locale=en",
                        "name": "Raw Goat",
                        "value": 2,
                        "identifier": "T4_MEAT"
                    }
                ]
            }
        },
        {
            "enchantment": 1,
            "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@1.png?quality=0&size=217&locale=en",
            "crafting": {
                "id": 70,
                "per_craft": 1,
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
                "requirements": [
                    {
                        "icon": "https://render.albiononline.com/v1/item/T5_FISH_SALTWATER_ALL_RARE@0.png?quality=0&size=217&locale=en",
                        "name": "Midwater Octopus",
                        "value": 1,
                        "identifier": "T5_FISH_SALTWATER_ALL_RARE"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_TURNIP@0.png?quality=0&size=217&locale=en",
                        "name": "Turnips",
                        "value": 2,
                        "identifier": "T4_TURNIP"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_BURDOCK@0.png?quality=0&size=217&locale=en",
                        "name": "Crenellated Burdock",
                        "value": 2,
                        "identifier": "T4_BURDOCK"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_MEAT@0.png?quality=0&size=217&locale=en",
                        "name": "Raw Goat",
                        "value": 2,
                        "identifier": "T4_MEAT"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T1_FISHSAUCE_LEVEL1@0.png?quality=0&size=217&locale=en",
                        "name": "Basic Fish Sauce",
                        "value": 9,
                        "identifier": "T1_FISHSAUCE_LEVEL1"
                    }
                ]
            }
        },
        {
            "enchantment": 2,
            "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@2.png?quality=0&size=217&locale=en",
            "crafting": {
                "id": 71,
                "per_craft": 1,
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
                "requirements": [
                    {
                        "icon": "https://render.albiononline.com/v1/item/T5_FISH_SALTWATER_ALL_RARE@0.png?quality=0&size=217&locale=en",
                        "name": "Midwater Octopus",
                        "value": 1,
                        "identifier": "T5_FISH_SALTWATER_ALL_RARE"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_TURNIP@0.png?quality=0&size=217&locale=en",
                        "name": "Turnips",
                        "value": 2,
                        "identifier": "T4_TURNIP"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_BURDOCK@0.png?quality=0&size=217&locale=en",
                        "name": "Crenellated Burdock",
                        "value": 2,
                        "identifier": "T4_BURDOCK"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_MEAT@0.png?quality=0&size=217&locale=en",
                        "name": "Raw Goat",
                        "value": 2,
                        "identifier": "T4_MEAT"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T1_FISHSAUCE_LEVEL2@0.png?quality=0&size=217&locale=en",
                        "name": "Fancy Fish Sauce",
                        "value": 9,
                        "identifier": "T1_FISHSAUCE_LEVEL2"
                    }
                ]
            }
        },
        {
            "enchantment": 3,
            "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@3.png?quality=0&size=217&locale=en",
            "crafting": {
                "id": 72,
                "per_craft": 1,
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
                "requirements": [
                    {
                        "icon": "https://render.albiononline.com/v1/item/T5_FISH_SALTWATER_ALL_RARE@0.png?quality=0&size=217&locale=en",
                        "name": "Midwater Octopus",
                        "value": 1,
                        "identifier": "T5_FISH_SALTWATER_ALL_RARE"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_TURNIP@0.png?quality=0&size=217&locale=en",
                        "name": "Turnips",
                        "value": 2,
                        "identifier": "T4_TURNIP"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_BURDOCK@0.png?quality=0&size=217&locale=en",
                        "name": "Crenellated Burdock",
                        "value": 2,
                        "identifier": "T4_BURDOCK"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T4_MEAT@0.png?quality=0&size=217&locale=en",
                        "name": "Raw Goat",
                        "value": 2,
                        "identifier": "T4_MEAT"
                    },
                    {
                        "icon": "https://render.albiononline.com/v1/item/T1_FISHSAUCE_LEVEL3@0.png?quality=0&size=217&locale=en",
                        "name": "Special Fish Sauce",
                        "value": 9,
                        "identifier": "T1_FISHSAUCE_LEVEL3"
                    }
                ]
            }
        }
    ]
}
```