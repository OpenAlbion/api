<?php

namespace App\Actions\ArmorSpell;

use App\Models\ArmorSpell;

class UpdateArmorSpell
{
    public function __construct(
        private ArmorSpell $model
    ) {
    }

    public function execute(array $item): ArmorSpell
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
            'spell_id' => data_get($item, 'spell_id'),
        ];
        $update = [];

        return [$search, $update];
    }
}
