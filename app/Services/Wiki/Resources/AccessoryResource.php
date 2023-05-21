<?php

namespace App\Services\Wiki\Resources;

use App\Services\Wiki\WikiService;
use Psr\Http\Message\ResponseInterface;

class AccessoryResource
{
    public function __construct(
        private readonly WikiService $service,
    ) {
    }

    public function capeList(): ResponseInterface
    {
        return $this->service->get(
            request: $this->service->buildRequest(),
            url: '/wiki/Cape',
        );
    }

    public function bagList(): ResponseInterface
    {
        return $this->service->get(
            request: $this->service->buildRequest(),
            url: '/wiki/Bag',
        );
    }
}
