<?php

namespace App\Http\Controllers\Api\V3;

use App\Http\Controllers\Controller;
use App\Http\Resources\V3\ConsumableStatResource;
use App\Models\ConsumableStat;
use App\Services\Render\RenderService;

class ConsumableStatController extends Controller
{
    public function __construct(
        private ConsumableStat $model
    ) {
    }

    public function byConsumableId($consumableId)
    {
        $data = cache()->remember(request()->generateCacheKey(), config('settings.cache_seconds'), function () use ($consumableId) {
            $stats = $this->model
                ->query()
                ->with('consumable')
                ->where('consumable_id', $consumableId)
                ->get();
            $statsResource = ConsumableStatResource::collection($stats)
                ->toArray(request());

            return [
                'data' => collect($statsResource)
                    ->groupBy('enchantment')
                    ->map(function ($group, $key) {
                        return [
                            'enchantment' => $key,
                            'icon' => app(RenderService::class)->setEnchantment($key)->renderItem($group->first()['consumable']->identifier),
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
