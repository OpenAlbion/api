<?php

namespace App\Actions\WeaponStat;

use App\Models\WeaponStat;

class UpdateWeaponStat
{
    public function __construct(
        private WeaponStat $model
    ) {
    }

    public function execute(array $item): WeaponStat
    {
        [$search, $update] = $this->toUpdateOrCrateArray($item);

        return $this->model
            ->query()
            ->updateOrCreate($search, $update);
    }

    public function toUpdateOrCrateArray(array $item): array
    {
        $search = [
            'weapon_id' => data_get($item, 'weapon_id'),
            'quality' => data_get($item, 'quality'),
            'enchantment' => data_get($item, 'enchantment'),
        ];
        $update = [
            'stats' => data_get($item, 'stats'),
        ];

        return [$search, $update];
    }
}
