<?php

namespace App\Http\Controllers\Api\Weaponry;

use App\Http\Controllers\Controller;
use App\Http\Resources\Weaponry\SearchResource;
use App\Models\Accessory;
use App\Models\Armor;
use App\Models\Consumable;
use App\Models\Weapon;
use Illuminate\Http\Request;

class SearchController extends Controller
{
    public function search(Request $request)
    {
        $data = cache()->remember(
            $request->generateCacheKey(),
            60,
            function () use ($request) {
                $weapons = Weapon::query()
                    ->where('name', 'like', '%'.$request->input('search').'%')
                    ->limit(3)
                    ->get();

                $armors = Armor::query()
                    ->where('name', 'like', '%'.$request->input('search').'%')
                    ->limit(3)
                    ->get();

                $accessories = Accessory::query()
                    ->where('name', 'like', '%'.$request->input('search').'%')
                    ->limit(3)
                    ->get();

                return [
                    ...SearchResource::collection($weapons)->toArray($request),
                    ...SearchResource::collection($armors)->toArray($request),
                    ...SearchResource::collection($accessories)->toArray($request),
                ];
            }
        );

        return response()->json(['data' => $data]);
    }

    public function v2Search(Request $request)
    {
        $data = cache()->remember(
            $request->generateCacheKey(),
            60,
            function () use ($request) {
                $weapons = Weapon::query()
                    ->where('name', 'like', '%'.$request->input('search').'%')
                    ->limit(3)
                    ->get();

                $armors = Armor::query()
                    ->where('name', 'like', '%'.$request->input('search').'%')
                    ->limit(3)
                    ->get();

                $accessories = Accessory::query()
                    ->where('name', 'like', '%'.$request->input('search').'%')
                    ->limit(3)
                    ->get();

                $consumables = Consumable::query()
                    ->where('name', 'like', '%'.$request->input('search').'%')
                    ->limit(3)
                    ->get();

                return [
                    ...SearchResource::collection($weapons)->toArray($request),
                    ...SearchResource::collection($armors)->toArray($request),
                    ...SearchResource::collection($accessories)->toArray($request),
                    ...SearchResource::collection($consumables)->toArray($request),
                ];
            }
        );

        return response()->json(['data' => $data]);
    }
}
