<?php

namespace App\Actions\Armor;

use App\Models\Armor;
use Illuminate\Support\Str;

class UpdateArmor
{
    public function __construct(
        private Armor $model
    ) {
    }

    public function execute(array $item): Armor
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
            'identifier' => data_get($item, 'name'),
            'path' => Str::wikiPath(data_get($item, 'path')),
            'tier' => data_get($item, 'tier'),
            'item_power' => Str::albionItemPower(data_get($item, 'item_power')),
        ];

        return [$search, $update];
    }
}
