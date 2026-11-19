<?php

namespace App\Actions\ConsumableStat;

use App\Models\ConsumableStat;

class UpdateConsumableStat
{
    public function __construct(
        private ConsumableStat $model
    ) {
    }

    public function execute(array $item): ConsumableStat
    {
        [$search, $update] = $this->toUpdateOrCrateArray($item);

        return $this->model
            ->query()
            ->updateOrCreate($search, $update);
    }

    public function toUpdateOrCrateArray(array $item): array
    {
        $search = [
            'consumable_id' => data_get($item, 'consumable_id'),
            'quality' => data_get($item, 'quality'),
            'enchantment' => data_get($item, 'enchantment'),
        ];
        $update = [
            'stats' => data_get($item, 'stats'),
        ];

        return [$search, $update];
    }
}
