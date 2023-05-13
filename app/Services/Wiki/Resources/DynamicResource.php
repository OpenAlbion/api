<?php

namespace App\Services\Wiki\Resources;

use App\Services\Wiki\WikiService;
use Psr\Http\Message\ResponseInterface;

class DynamicResource
{
    public function __construct(
        private readonly WikiService $service,
    ) {
    }

    public function get(string $path): ResponseInterface
    {
        return $this->service->get(
            request: $this->service->buildRequest(),
            url: $path,
        );
    }
}
