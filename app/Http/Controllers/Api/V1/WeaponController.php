<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Resources\V1\WeaponResource;
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
        $data = $this->model
            ->query()
            ->when($request->input('category_id'), function ($query, $category) {
                $query->where('category_id', $category);
            })
            ->when($request->input('subcategory_id'), function ($query, $category) {
                $query->where('subcategory_id', $category);
            })
            ->when($request->input('tire'), function ($query, $tire) {
                $query->where('tire', $tire);
            })
            ->get();

        return WeaponResource::collection($data);
    }
}
