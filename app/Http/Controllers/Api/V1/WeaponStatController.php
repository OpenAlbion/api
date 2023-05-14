<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Resources\V1\WeaponStatResource;
use App\Models\WeaponStat;
use App\Services\Render\RenderService;

class WeaponStatController extends Controller
{
    public function __construct(
        private WeaponStat $model
    ) {
    }

    public function byWeaponId($weaponId)
    {
        $data = $this->model
            ->query()
            ->with('weapon')
            ->where('weapon_id', $weaponId)
            ->get();
        $data = WeaponStatResource::collection($data)
            ->toArray(request());

        return response()->json([
            'data' => collect($data)
                ->groupBy('enchantment')
                ->map(function ($group, $key) {
                    return [
                        'enchantment' => $key,
                        'icon' => app(RenderService::class)->setEnchantment($key)->renderItem($group->first()['weapon']->identifier),
                        'stats' => $group,
                    ];
                })
                ->values()
                ->toArray(request()),
        ]);
    }
}
