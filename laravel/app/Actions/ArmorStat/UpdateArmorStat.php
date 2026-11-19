<?php

namespace App\Actions\ArmorStat;

use App\Models\ArmorStat;

class UpdateArmorStat
{
    public function __construct(
        private ArmorStat $model
    ) {
    }

    public function execute(array $item): ArmorStat
    {
        [$search, $update] = $this->toUpdateOrCrateArray($item);

        return $this->model
            ->query()
            ->updateOrCreate($search, $update);
    }

    public function toUpdateOrCrateArray(array $item): array
    {
        $search = [
            'armor_id' => data_get($item, 'armor_id'),
            'quality' => data_get($item, 'quality'),
            'enchantment' => data_get($item, 'enchantment'),
        ];
        $update = [
            'stats' => data_get($item, 'stats'),
        ];

        return [$search, $update];
    }
}
