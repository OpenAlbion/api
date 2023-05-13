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
        $data = $this->model
            ->query()
            ->whereHas('weaponSpells', function ($query) use ($weaponId) {
                $query->where('weapon_id', $weaponId);
            })
            ->get();
        $data = SpellResource::collection($data)
            ->toArray(request());

        return response()->json([
            'data' => collect($data)
                ->groupBy('slot')
                ->map(function ($group, $key) {
                    return [
                        'slot' => $key,
                        'spells' => $group,
                    ];
                })
                ->values()
                ->toArray(request()),
        ]);
    }
}
