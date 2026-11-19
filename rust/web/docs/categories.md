# Categories

## Get Categories For Weapon

Retrieve information about OpenAlbion's categories for weapon.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/categories**
- Query Parameters:
  - `type`: Filter the categories by type. Example: `weapon`

Example Request:
```http
GET https://api.openalbion.com/api/v3/categories?type=weapon
```

Example Response:
```json
{
    "data": [
        {
            "id": 1,
            "name": "Warrior Weapons",
            "type": "weapon",
            "subcategories": [
                {
                    "id": 2,
                    "name": "Axe",
                    "type": "weapon"
                },
                {
                    "id": 3,
                    "name": "Sword",
                    "type": "weapon"
                },
                {...}
            ]
        },
        {...}
    ]
}
```

## Get Categories For Armor

Retrieve information about OpenAlbion's categories for armor.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/categories**
- Query Parameters:
  - `type`: Filter the categories by type. Example: `armor`

Example Request:
```http
GET https://api.openalbion.com/api/v3/categories?type=armor
```

Example Response:
```json
{
    "data": [
        {
            "id": 23,
            "name": "Cloth Armor",
            "type": "armor",
            "subcategories": [
                {
                    "id": 24,
                    "name": "Robes",
                    "type": "armor"
                },
                {...}
            ]
        },
        {...}
    ]
}
```

## Get Categories For Accessory

- Cape
- Bag
- Mount

There are main three categories for accessory. 

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/categories**
- Query Parameters:
  - `type`: Filter the categories by type. Example: `accessory`

Example Request:
```http
GET https://api.openalbion.com/api/v3/categories?type=accessory
```

Example Response:
```json
{
    "data": [
        {
            "id": 35,
            "name": "Cape",
            "type": "accessory",
            "subcategories": [
                {
                    "id": 37,
                    "name": "Normal Capes",
                    "type": "accessory"
                },
                {
                    "id": 38,
                    "name": "Faction Capes",
                    "type": "accessory"
                }
            ]
        },
        {
            "id": 36,
            "name": "Bag",
            "type": "accessory",
            "subcategories": [
                {
                    "id": 39,
                    "name": "Normal Bags",
                    "type": "accessory"
                }
            ]
        },
        {
            "id": 40,
            "name": "Mount",
            "type": "accessory",
            "subcategories": [
                {
                    "id": 41,
                    "name": "Armored Horses",
                    "type": "accessory"
                },
                {
                    "id": 42,
                    "name": "Mules",
                    "type": "accessory"
                },
                {...}
            ]
        }
    ]
}
```

## Get Categories For Consumable

- Food
- Potion

There are main two categories for consumable. 

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/categories**
- Query Parameters:
  - `type`: Filter the categories by type. Example: `consumable`

Example Request:
```http
GET https://api.openalbion.com/api/v3/categories?type=consumable
```

Example Response:
```json
{
    "data": [
        {
            "id": 52,
            "name": "Foods",
            "type": "consumable",
            "subcategories": [
                {
                    "id": 53,
                    "name": "Omelette",
                    "type": "consumable"
                },
                {
                    "id": 59,
                    "name": "Roast",
                    "type": "consumable"
                },
                {...}
            ]
        },
        {
            "id": 60,
            "name": "Potions",
            "type": "consumable",
            "subcategories": [
                {
                    "id": 61,
                    "name": "Energy",
                    "type": "consumable"
                },
                {
                    "id": 68,
                    "name": "Focus",
                    "type": "consumable"
                },
                {...}
            ]
        }
    ]
}
```