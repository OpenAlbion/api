<?php

namespace App\Search;

use Algolia\ScoutExtended\Searchable\Aggregator;
use App\Models\Accessory;
use App\Models\Armor;
use App\Models\Consumable;
use App\Models\Weapon;

class Items extends Aggregator
{
    /**
     * The names of the models that should be aggregated.
     *
     * @var string[]
     */
    protected $models = [
        Weapon::class,
        Armor::class,
        Accessory::class,
        Consumable::class,
    ];
}
