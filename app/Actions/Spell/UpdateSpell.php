<?php

namespace App\Actions\Spell;

use App\Models\Spell;
use Illuminate\Support\Str;

class UpdateSpell
{
    public function __construct(
        private Spell $model
    ) {
    }

    public function execute(array $item): Spell
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
            'identifier' => Str::albionIdentifier(data_get($item, 'icon')),
            'slot' => data_get($item, 'slot'),
            'attributes' => data_get($item, 'attributes'),
            'description' => Str::sanitizeSpellDescription(data_get($item, 'description')),
            'preview' => data_get($item, 'preview'),
            'ref' => data_get($item, 'ref'),
        ];

        return [$search, $update];
    }
}
