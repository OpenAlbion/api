<?php

namespace App\Actions\Weapon;

use App\Models\Weapon;
use Illuminate\Support\Str;

class UpdateWeapon
{
    public function __construct(
        private Weapon $model
    ) {
    }

    public function execute(array $item): Weapon
    {
        [$search, $update] = $this->toUpdateOrCrateArray($item);

        return $this->model
            ->query()
            ->updateOrCreate($search, $update);
    }

    public function toUpdateOrCrateArray(array $item): array
    {
        $search = [
            'name' => data_get($item, 'name'),
        ];
        $update = [
            'category_id' => data_get($item, 'category_id'),
            'subcategory_id' => data_get($item, 'subcategory_id'),
            'identifier' => Str::albionIdentifier(data_get($item, 'icon')),
            'path' => Str::wikiPath(data_get($item, 'path')),
            'tier' => data_get($item, 'tier'),
            'item_power' => Str::albionItemPower(data_get($item, 'item_power')),
        ];

        return [$search, $update];
    }
}
