<?php

namespace App\Http\Controllers\Api\V2;

use App\Http\Controllers\Controller;
use App\Http\Resources\V2\CategoryResource;
use App\Models\Category;
use Illuminate\Http\Request;

class CategoryController extends Controller
{
    public function __construct(
        private Category $model
    ) {
    }

    public function index(Request $request)
    {
        $data = cache()->remember($request->generateCacheKey(), config('settings.cache_seconds'), function () use ($request) {
            return $this->model
                ->query()
                ->with('children')
                ->where('parent_id', null)
                ->where('version', '<=', 2)
                ->when($request->input('type'), function ($query, $type) {
                    $query->where('type', $type);
                })
                ->get();
        });

        return CategoryResource::collection($data);
    }
}
