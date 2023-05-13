<?php

namespace App\Actions\Category;

use App\Models\Category;
use Illuminate\Support\Str;

class UpdateCategory
{
    public function __construct(
        private Category $model
    ) {
    }

    public function execute(array $item): Category
    {
        [$search, $update] = $this->toUpdateOrCrateArray($item);

        return $this->model
            ->query()
            ->updateOrCreate($search, $update);
    }

    public function toUpdateOrCrateArray(array $item): array
    {
        $search = [
            'name' => data_get($item, 'name'),
        ];
        $update = [
            'parent_id' => data_get($item, 'parent_id'),
            'path' => Str::wikiPath(data_get($item, 'path')),
            'type' => data_get($item, 'type'),
        ];

        return [$search, $update];
    }
}
