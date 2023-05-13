<?php

namespace App\Services\Wiki\Resources;

use App\Services\Wiki\WikiService;
use Psr\Http\Message\ResponseInterface;

class WeaponResource
{
    public function __construct(
        private readonly WikiService $service,
    ) {
    }

    public function categoryList(): ResponseInterface
    {
        return $this->service->get(
            request: $this->service->buildRequest(),
            url: '/wiki/Weapon',
        );
    }
}
