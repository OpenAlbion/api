<?php

namespace App\Services\Wiki\Resources;

use App\Services\Wiki\Contracts\WikiResponseInterface;
use App\Services\Wiki\WikiService;

class DynamicResource
{
    public function __construct(
        private readonly WikiService $service,
    ) {
    }

    public function get(string $path): WikiResponseInterface
    {
        return $this->service->get(
            request: $this->service->buildRequest(),
            url: $path,
        );
    }
}
