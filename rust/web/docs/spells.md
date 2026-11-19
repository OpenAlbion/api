# Spells

## Get Spells By Weapon

Retrieve information about spells by weapon.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/spells/weapon/{weapon_id}**

Example Request:
```http
GET https://api.openalbion.com/api/v3/spells/weapon/1
```

Example Response:
```json
{
    "data": [
        {
            "slot": "First Slot",
            "spells": [
                {
                    "id": 1,
                    "name": "Rending Strike",
                    "slot": "First Slot",
                    "preview": "https://wiki.albiononline.com/data/images/d/d4/Rending_Strike.gif",
                    "icon": "https://render.albiononline.com/v1/spell/Rending Strike.png?size=217&locale=en",
                    "attributes": [
                        {
                            "name": "Energy Cost",
                            "value": "6"
                        },
                        {
                            "name": "Cast Time",
                            "value": "Instant"
                        },
                        {
                            "name": "Range",
                            "value": "3m"
                        },
                        {
                            "name": "Cooldown",
                            "value": "2s"
                        }
                    ],
                    "description": "Attack the targeted enemy, dealing 178/196 physical damage.\nApplies one Rending Bleed Charge.\nEach Rending Bleed Charge deals 83/91 over 6s (stacks up to 3 times).\nIf the maximum amount of stacks is reached, the affected enemies' Healing Received is reduced by 12% for 5s.\n\n",
                    "description_html": "Attack the targeted enemy, dealing <span ><i>178</i>/<i>196</i> physical damage</span>.\n<p>Applies one <b>Rending Bleed Charge</b>.\n</p><p>Each <b>Rending Bleed Charge</b> deals <span ><i>83</i>/<i>91</i></span> over <span >6s</span> (<b>stacks</b> up to <b>3</b> times).\n</p><p>If the <b>maximum amount</b> of <b>stacks</b> is reached, the affected enemies' <span >Healing Received</span> is reduced by <span >12%</span> for <span >5s</span>.\n</p>\n"
                },
                {
                    "id": 2,
                    "name": "Rending Spin",
                    "slot": "First Slot",
                    "preview": null,
                    "icon": "https://render.albiononline.com/v1/spell/Rending Spin.png?size=217&locale=en",
                    "attributes": [
                        {
                            "name": "Energy Cost",
                            "value": "10"
                        },
                        {
                            "name": "Cast Time",
                            "value": "instant"
                        },
                        {
                            "name": "Range",
                            "value": "self"
                        },
                        {
                            "name": "Cooldown",
                            "value": "3s"
                        }
                    ],
                    "description": "Swing the weapon around you dealing physical damage to all enemies in a 6m radius around you. Deals 248 to enemies that are at least 3m away and 149 to enemies that are closer than that. Also inflicts a Rending Bleed to each enemy hit, which lasts for 6s and deals 18 physical damage per second. (Stacks up to 3 times.)\nIf you build up 3 stacks, enemies hit will have their healing received reduced by 20% for 5s.\n\n",
                    "description_html": "Swing the weapon around you dealing physical damage to all enemies in a 6m radius around you. Deals <b>248</b> to enemies that are at least 3m away and <b>149</b> to enemies that are closer than that. Also inflicts a Rending Bleed to each enemy hit, which lasts for 6s and deals <b>18</b> physical damage per second. (Stacks up to 3 times.)\n<p>If you build up 3 stacks, enemies hit will have their healing received reduced by 20% for 5s.\n</p>\n"
                }
            ]
        },
        {
            "slot": "Second Slot",
            "spells": [
                {...}
            ]
        },
        {
            "slot": "Third Slot",
            "spells": [
                {...}
            ]
        },
        {
            "slot": "Passive",
            "spells": [
                {...}
            ]
        }
    ]
}
```

## Get Spells By Armor

Retrieve information about spells by armor.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/spells/armor/{armor_id}**

Example Request:
```http
GET https://api.openalbion.com/api/v3/spells/armor/1
```

Example Response:
```json
{
    "data": [
        {
            "slot": "Fifth Slot",
            "spells": [
                {
                    "id": 261,
                    "name": "Mend Wounds",
                    "slot": "Fifth Slot",
                    "preview": null,
                    "icon": "https://render.albiononline.com/v1/spell/OUTOFCOMBATHEAL.png?size=217&locale=en",
                    "attributes": [
                        {
                            "name": "Energy Cost",
                            "value": "0"
                        },
                        {
                            "name": "Cast Time",
                            "value": "channeled"
                        },
                        {
                            "name": "Range",
                            "value": "self"
                        },
                        {
                            "name": "Cooldown",
                            "value": "30s"
                        }
                    ],
                    "description": "While not engaged in combat, rest while channeling for 12s.\nRestore 6% of your Max Health (not affected by your Healing Cast modifier) and 5% of your Max Energy every 1s.\n\n",
                    "description_html": "While <b>not</b> engaged in <b>combat</b>, rest while <b>channeling</b> for <b>12s</b>.\n<p>Restore <span >6%</span> of your <span >Max Health</span> (not affected by your Healing Cast modifier) and <b>5%</b> of your <b>Max Energy</b> every <b>1s</b>.\n</p>\n"
                },
                {...}
            ]
        },
        {
            "slot": "Passive",
            "spells": [
                {...}
            ]
        }
    ]
}
```

## Get Spells By Accessory

Retrieve information about spells by accessory.

- Method: GET
- Endpoint: **https://api.openalbion.com/api/v3/spells/accessory/{accessory_id}**

Example Request:
```http
GET https://api.openalbion.com/api/v3/spells/accessory/3
```

Example Response:
```json
{
    "data": [
        {
            "slot": "Passive",
            "spells": [
                {
                    "id": 360,
                    "name": "Sandstorm",
                    "slot": "Passive",
                    "preview": null,
                    "icon": "https://render.albiononline.com/v1/spell/Sandstorm.png?size=217&locale=en",
                    "attributes": [
                        {
                            "name": "Condition:",
                            "value": "Activates when you use your third-slot ability."
                        },
                        {
                            "name": "Effect",
                            "value": "Create a sandstorm in a 4m radius around you, lasting 6s. Slow all enemies within it by 20%. Decreases Crowd Control Resistance by 53."
                        },
                        {
                            "name": "Cooldown",
                            "value": "58s"
                        }
                    ],
                    "description": "Sandstorm is a passive ability found on the Bridgewatch Cape.\n",
                    "description_html": "Sandstorm is a passive ability found on the <b><a href=\"/wiki/Bridgewatch_Cape\"  title=\"Bridgewatch Cape\">Bridgewatch Cape.</a></b>\n"
                }
            ]
        }
    ]
}
```