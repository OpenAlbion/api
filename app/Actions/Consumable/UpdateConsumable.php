<?php

namespace App\Actions\Consumable;

use App\Models\Consumable;
use Illuminate\Support\Str;

class UpdateConsumable
{
    public function __construct(
        private Consumable $model
    ) {
    }

    public function execute(array $item): Consumable
    {
        [$search, $update] = $this->toUpdateOrCrateArray($item);

        return $this->model
            ->query()
            ->updateOrCreate($search, $update);
    }

    public function toUpdateOrCrateArray(array $item): array
    {
        $search = [
            'name' => Str::replace('  ', '', data_get($item, 'name')),
        ];
        $update = [
            'category_id' => data_get($item, 'category_id'),
            'subcategory_id' => data_get($item, 'subcategory_id'),
            'identifier' => null,
            'path' => Str::wikiPath(data_get($item, 'path')),
            'tier' => data_get($item, 'tier'),
            'item_power' => Str::albionItemPower(data_get($item, 'item_power')),
        ];

        return [$search, $update];
    }
}
