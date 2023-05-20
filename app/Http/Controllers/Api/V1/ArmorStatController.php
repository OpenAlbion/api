<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Resources\V1\ArmorStatResource;
use App\Models\ArmorStat;
use App\Services\Render\RenderService;
use Illuminate\Http\Request;

class ArmorStatController extends Controller
{
    public function __construct(
        private ArmorStat $model
    ) {
    }

    public function byArmorId($armorId)
    {
        $data = cache()->remember(request()->generateCacheKey(), config('settings.cache_seconds'), function () use ($armorId) {
            $stats = $this->model
                ->query()
                ->with('armor')
                ->where('armor_id', $armorId)
                ->get();
            $statsResource = ArmorStatResource::collection($stats)
                ->toArray(request());

            return [
                'data' => collect($statsResource)
                    ->groupBy('enchantment')
                    ->map(function ($group, $key) {
                        return [
                            'enchantment' => $key,
                            'icon' => app(RenderService::class)->setEnchantment($key)->renderItem($group->first()['armor']->identifier),
                            'stats' => $group,
                        ];
                    })
                    ->values()
                    ->toArray(request()),
            ];
        });

        return response()->json($data);
    }
}
