<?php

namespace App\Http\Controllers\Api\V3;

use App\Http\Controllers\Controller;
use App\Http\Resources\V3\SpellResource;
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

    public function byArmorId($armorId)
    {
        $data = cache()->remember(request()->generateCacheKey(), config('settings.cache_seconds'), function () use ($armorId) {
            $spells = $this->model
                ->query()
                ->whereHas('armorSpells', function ($query) use ($armorId) {
                    $query->where('armor_id', $armorId);
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

    public function byAccessoryId($accessoryId)
    {
        $data = cache()->remember(request()->generateCacheKey(), config('settings.cache_seconds'), function () use ($accessoryId) {
            $spells = $this->model
                ->query()
                ->whereHas('accessorySpells', function ($query) use ($accessoryId) {
                    $query->where('accessory_id', $accessoryId);
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
