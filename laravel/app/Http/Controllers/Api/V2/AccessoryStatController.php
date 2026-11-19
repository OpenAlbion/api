<?php

namespace App\Http\Controllers\Api\V2;

use App\Http\Controllers\Controller;
use App\Http\Resources\V2\AccessoryStatResource;
use App\Models\AccessoryStat;
use App\Services\Render\RenderService;

class AccessoryStatController extends Controller
{
    public function __construct(
        private AccessoryStat $model
    ) {
    }

    public function byAccessoryId($accessoryId)
    {
        $data = cache()->remember(request()->generateCacheKey(), config('settings.cache_seconds'), function () use ($accessoryId) {
            $stats = $this->model
                ->query()
                ->with('accessory')
                ->where('accessory_id', $accessoryId)
                ->get();
            $statsResource = AccessoryStatResource::collection($stats)
                ->toArray(request());

            return [
                'data' => collect($statsResource)
                    ->groupBy('enchantment')
                    ->map(function ($group, $key) {
                        return [
                            'enchantment' => $key,
                            'icon' => app(RenderService::class)->setEnchantment($key)->renderItem($group->first()['accessory']->identifier),
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
