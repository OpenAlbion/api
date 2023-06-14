<?php

namespace App\Enums;

enum CategoryType: string
{
    case WEAPON = 'weapon';
    case ARMOR = 'armor';
    case ACCESSORY = 'accessory';
    case MOUNT = 'mount';
    case CONSUMABLE = 'consumable';
}
