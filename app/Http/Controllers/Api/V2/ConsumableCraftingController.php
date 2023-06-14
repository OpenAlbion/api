<?php

namespace App\Http\Controllers\Api\V2;

use App\Http\Controllers\Controller;
use App\Http\Resources\V2\ConsumableCraftingResource;
use App\Models\ConsumableCrafting;
use App\Services\Render\RenderService;

class ConsumableCraftingController extends Controller
{
    public function __construct(
        private ConsumableCrafting $model
    ) {
    }

    public function byConsumableId($consumableId)
    {
        $data = cache()->remember(request()->generateCacheKey() . 'test', 0, function () use ($consumableId) {
            $craftings = $this->model
                ->query()
                ->with('consumable')
                ->where('consumable_id', $consumableId)
                ->get();
            $craftingsResource = ConsumableCraftingResource::collection($craftings)
                ->toArray(request());

            return [
                'data' => collect($craftingsResource)
                    ->groupBy('enchantment')
                    ->map(function ($group, $key) {
                        return [
                            'enchantment' => $key,
                            'icon' => app(RenderService::class)->setEnchantment($key)->renderItem($group->first()['consumable']->identifier),
                            'craftings' => $group->first(),
                        ];
                    })
                    ->values()
                    ->toArray(request()),
            ];
        });

        return response()->json($data);
    }
}
