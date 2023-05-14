<?php

namespace App\Http\Controllers\Api\V1;

use App\Http\Controllers\Controller;
use App\Http\Resources\V1\CategoryResource;
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
        $data = cache()->remember($request->generateCacheKey(), 180, function () {
            return $this->model
                ->query()
                ->with('children')
                ->where('parent_id', null)
                ->when(request()->input('type'), function ($query, $type) {
                    $query->where('type', $type);
                })
                ->get();
        });

        return CategoryResource::collection($data);
    }
}
