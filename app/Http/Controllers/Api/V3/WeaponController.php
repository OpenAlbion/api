<?php

namespace App\Http\Controllers\Api\V3;

use App\Http\Controllers\Controller;
use App\Http\Resources\V3\WeaponResource;
use App\Models\Weapon;
use Illuminate\Http\Request;

class WeaponController extends Controller
{
    public function __construct(
        private Weapon $model
    ) {
    }

    public function index(Request $request)
    {
        $data = cache()->remember($request->generateCacheKey(), config('settings.cache_seconds'), function () use ($request) {
            return $this->model
                ->query()
                ->when($request->input('category_id'), function ($query, $category) {
                    $query->where('category_id', $category);
                })
                ->when($request->input('subcategory_id'), function ($query, $category) {
                    $query->where('subcategory_id', $category);
                })
                ->when($request->input('tier'), function ($query, $tier) {
                    $query->where('tier', $tier);
                })
                ->get();
        });

        return WeaponResource::collection($data);
    }
}
