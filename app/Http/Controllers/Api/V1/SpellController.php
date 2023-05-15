<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Resources\V1\SpellResource;
use App\Models\Spell;

class SpellController extends Controller
{
    public function __construct(
        private Spell $model
    ) {
    }

    public function byWeaponId($weaponId)
    {
        $data = cache()->remember(request()->generateCacheKey(), config('settings.cache_seconds'), function () use ($weaponId) {
            $spells = $this->model
                ->query()
                ->whereHas('weaponSpells', function ($query) use ($weaponId) {
                    $query->where('weapon_id', $weaponId);
                })
                ->get();
            $spellsResource = SpellResource::collection($spells)
                ->toArray(request());

            return [
                'data' => collect($spellsResource)
                    ->groupBy('slot')
                    ->map(function ($group, $key) {
                        return [
                            'slot' => $key,
                            'spells' => $group,
                        ];
                    })
                    ->values()
                    ->toArray(request()),
            ];
        });

        return response()->json($data);
    }
}
