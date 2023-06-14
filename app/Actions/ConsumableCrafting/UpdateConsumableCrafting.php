<?php

namespace App\Actions\ConsumableCrafting;

use App\Models\ConsumableCrafting;

class UpdateConsumableCrafting
{
    public function __construct(
        private ConsumableCrafting $model
    ) {
    }

    public function execute(array $item): ConsumableCrafting
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
            'per_craft' => data_get($item, 'per_craft'),
            'enchantment' => data_get($item, 'enchantment'),
        ];
        $update = [
            'requirements' => data_get($item, 'requirements'),
        ];

        return [$search, $update];
    }
}
