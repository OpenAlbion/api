# Consumables

## Get All Consumables

Retrieve information about consumables.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/consumables**

Example Request:
```http
GET https://api.openalbion.com/api/v3/consumables
```

Example Response:
```json
{
    "data": [
        {
            "id": 1,
            "name": "Avalonian Chicken Omelette",
            "tier": "3.0",
            "item_power": 600,
            "identifier": "T3_MEAL_OMELETTE_AVALON",
            "icon": "https://render.albiononline.com/v1/item/T3_MEAL_OMELETTE_AVALON@0.png?quality=0&size=217&locale=en",
            "info": "Players may craft Avalonian Chicken Omelette at the Cook\nAvalonian Chicken Omelette may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Avalonian Chicken Omelette last for 30 minutes\nAvalonian Chicken Omelette per craft: 10\n",
            "category": { "id": 52, "name": "Foods", "type": "consumable" },
            "subcategory": { "id": 53, "name": "Omelette", "type": "consumable" }
        },
        {
            "id": 2,
            "name": "Avalonian Goose Omelette",
            "tier": "5.0",
            "item_power": 800,
            "identifier": "T5_MEAL_OMELETTE_AVALON",
            "icon": "https://render.albiononline.com/v1/item/T5_MEAL_OMELETTE_AVALON@0.png?quality=0&size=217&locale=en",
            "info": "Players may craft Avalonian Goose Omelette at the Cook\nAvalonian Goose Omelette may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Avalonian Goose Omelette last for 30 minutes\nAvalonian Goose Omelette per craft: 10\n",
            "category": { "id": 52, "name": "Foods", "type": "consumable" },
            "subcategory": { "id": 53, "name": "Omelette", "type": "consumable" }
        },
        {...}
    ]
}
```

## Get Consumables By Category

Retrieve information about consumables by category.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/consumables**
- Query Parameters:
  - `category_id`: Filter the consumables by category. Example: `52`

Example Request:
```http
GET https://api.openalbion.com/api/v3/consumables?category_id=52
```

Example Response:
```json
{
    "data": [
        {
            "id": 1,
            "name": "Avalonian Chicken Omelette",
            "tier": "3.0",
            "item_power": 600,
            "identifier": "T3_MEAL_OMELETTE_AVALON",
            "icon": "https://render.albiononline.com/v1/item/T3_MEAL_OMELETTE_AVALON@0.png?quality=0&size=217&locale=en",
            "info": "Players may craft Avalonian Chicken Omelette at the Cook\nAvalonian Chicken Omelette may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Avalonian Chicken Omelette last for 30 minutes\nAvalonian Chicken Omelette per craft: 10\n",
            "category": { "id": 52, "name": "Foods", "type": "consumable" },
            "subcategory": { "id": 53, "name": "Omelette", "type": "consumable" }
        },
        {
            "id": 2,
            "name": "Avalonian Goose Omelette",
            "tier": "5.0",
            "item_power": 800,
            "identifier": "T5_MEAL_OMELETTE_AVALON",
            "icon": "https://render.albiononline.com/v1/item/T5_MEAL_OMELETTE_AVALON@0.png?quality=0&size=217&locale=en",
            "info": "Players may craft Avalonian Goose Omelette at the Cook\nAvalonian Goose Omelette may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Avalonian Goose Omelette last for 30 minutes\nAvalonian Goose Omelette per craft: 10\n",
            "category": { "id": 52, "name": "Foods", "type": "consumable" },
            "subcategory": { "id": 53, "name": "Omelette", "type": "consumable" }
        },
        {...}
    ]
}
```

## Get Consumables By Subcategory

Retrieve information about consumables by subcategory.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/consumables**
- Query Parameters:
  - `subcategory_id`: Filter the consumables by subcategory. Example: `53`

Example Request:
```http
GET https://api.openalbion.com/api/v3/consumables?subcategory_id=53
```

Example Response:
```json
{
    "data": [
        {
            "id": 1,
            "name": "Avalonian Chicken Omelette",
            "tier": "3.0",
            "item_power": 600,
            "identifier": "T3_MEAL_OMELETTE_AVALON",
            "icon": "https://render.albiononline.com/v1/item/T3_MEAL_OMELETTE_AVALON@0.png?quality=0&size=217&locale=en",
            "info": "Players may craft Avalonian Chicken Omelette at the Cook\nAvalonian Chicken Omelette may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Avalonian Chicken Omelette last for 30 minutes\nAvalonian Chicken Omelette per craft: 10\n",
            "category": { "id": 52, "name": "Foods", "type": "consumable" },
            "subcategory": { "id": 53, "name": "Omelette", "type": "consumable" }
        },
        {
            "id": 2,
            "name": "Avalonian Goose Omelette",
            "tier": "5.0",
            "item_power": 800,
            "identifier": "T5_MEAL_OMELETTE_AVALON",
            "icon": "https://render.albiononline.com/v1/item/T5_MEAL_OMELETTE_AVALON@0.png?quality=0&size=217&locale=en",
            "info": "Players may craft Avalonian Goose Omelette at the Cook\nAvalonian Goose Omelette may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Avalonian Goose Omelette last for 30 minutes\nAvalonian Goose Omelette per craft: 10\n",
            "category": { "id": 52, "name": "Foods", "type": "consumable" },
            "subcategory": { "id": 53, "name": "Omelette", "type": "consumable" }
        },
        {...}
    ]
}
```

## Get Consumables By Tier

Retrieve information about consumables by tier.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/consumables**
- Query Parameters:
  - `tier`: Filter the consumables by tier. Example: `4`

Example Request:
```http
GET https://api.openalbion.com/api/v3/consumables?tier=4
```

Example Response:
```json
{
    "data": [
        {
            "id": 18,
            "name": "Midwater Octopus Salad",
            "tier": "4.0",
            "item_power": 700,
            "identifier": "T4_MEAL_SALAD_FISH",
            "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD_FISH@0.png?quality=0&size=217&locale=en",
            "info": "Players may craft Midwater Octopus Salad at the Cook\nMidwater Octopus Salad may add nutrition to crafting stations\nPlayers may only have one type of Food active at one time\nEffects of Midwater Octopus Salad last for 2 minutes\nMidwater Octopus Salad per craft: 1\n",
            "category": { "id": 52, "name": "Foods", "type": "consumable" },
            "subcategory": { "id": 55, "name": "Salad", "type": "consumable" }
        },
        {
            "id": 22,
            "name": "Turnip Salad",
            "tier": "4.0",
            "item_power": 700,
            "identifier": "T4_MEAL_SALAD",
            "icon": "https://render.albiononline.com/v1/item/T4_MEAL_SALAD@0.png?quality=0&size=217&locale=en",
            "info": "Players may craft Turnip Salad at the Cook\nTurnip Salad may add nutrition to crafting stations and is the Favorite Dish at the Toolmaker\nPlayers may only have one type of Food active at one time\nEffects of Turnip Salad last for 30 minutes\nTurnip Salad per craft: 10 ( takes 2.16 nutrition to make 10 based on the equation: (item value)*.01125 )\n",
            "category": { "id": 52, "name": "Foods", "type": "consumable" },
            "subcategory": { "id": 55, "name": "Salad", "type": "consumable" }
        },
        {...}
    ]
}
```