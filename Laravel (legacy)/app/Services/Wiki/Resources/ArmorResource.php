<?php

namespace App\Services\Wiki\Resources;

use App\Services\Wiki\Contracts\WikiResponseInterface;
use App\Services\Wiki\WikiService;

class ArmorResource
{
    public function __construct(
        private readonly WikiService $service,
    ) {
    }

    public function categoryList(): WikiResponseInterface
    {
        return $this->service->get(
            request: $this->service->buildRequest(),
            url: '/wiki/Armor',
        );
    }
}
