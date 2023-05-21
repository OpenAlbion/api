<?php

namespace App\Actions\AccessorySpell;

use App\Models\AccessorySpell;

class UpdateAccessorySpell
{
    public function __construct(
        private AccessorySpell $model
    ) {
    }

    public function execute(array $item): AccessorySpell
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
            'spell_id' => data_get($item, 'spell_id'),
        ];
        $update = [];

        return [$search, $update];
    }
}
