<?php

namespace App\Actions\AccessoryStat;

use App\Models\AccessoryStat;

class UpdateAccessoryStat
{
    public function __construct(
        private AccessoryStat $model
    ) {
    }

    public function execute(array $item): AccessoryStat
    {
        [$search, $update] = $this->toUpdateOrCrateArray($item);

        return $this->model
            ->query()
            ->updateOrCreate($search, $update);
    }

    public function toUpdateOrCrateArray(array $item): array
    {
        $search = [
            'accessory_id' => data_get($item, 'accessory_id'),
            'quality' => data_get($item, 'quality'),
            'enchantment' => data_get($item, 'enchantment'),
        ];
        $update = [
            'stats' => data_get($item, 'stats'),
        ];

        return [$search, $update];
    }
}
