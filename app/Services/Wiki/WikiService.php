<?php

namespace App\Services\Wiki;

use App\Services\Concerns\BuildRequest;
use App\Services\Concerns\CanSendGetRequest;
use App\Services\Wiki\Resources\DynamicResource;
use App\Services\Wiki\Resources\WeaponResource;

class WikiService
{
    use BuildRequest;
    use CanSendGetRequest;

    public function __construct(
        private readonly string $baseUrl
    ) {
    }

    public function weapon(): WeaponResource
    {
        return new WeaponResource(
            service: $this,
        );
    }

    public function dynamic(): DynamicResource
    {
        return new DynamicResource(
            service: $this,
        );
    }
}
