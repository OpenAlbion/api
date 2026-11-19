<?php

namespace App\Services\Wiki\Resources;

use App\Services\Wiki\Contracts\WikiResponseInterface;
use App\Services\Wiki\WikiService;

class AccessoryResource
{
    public function __construct(
        private readonly WikiService $service,
    ) {
    }

    public function capeList(): WikiResponseInterface
    {
        return $this->service->get(
            request: $this->service->buildRequest(),
            url: '/wiki/Cape',
        );
    }

    public function bagList(): WikiResponseInterface
    {
        return $this->service->get(
            request: $this->service->buildRequest(),
            url: '/wiki/Bag',
        );
    }

    public function mountList(): WikiResponseInterface
    {
        return $this->service->get(
            request: $this->service->buildRequest(),
            url: '/wiki/Adept%27s_Giant_Stag',
        );
    }
}
