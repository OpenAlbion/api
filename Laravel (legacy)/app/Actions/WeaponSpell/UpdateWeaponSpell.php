<?php

namespace App\Actions\WeaponSpell;

use App\Models\WeaponSpell;

class UpdateWeaponSpell
{
    public function __construct(
        private WeaponSpell $model
    ) {
    }

    public function execute(array $item): WeaponSpell
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
            'spell_id' => data_get($item, 'spell_id'),
        ];
        $update = [];

        return [$search, $update];
    }
}
