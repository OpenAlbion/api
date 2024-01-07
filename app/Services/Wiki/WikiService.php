<?php

namespace App\Services\Wiki;

use App\Services\Wiki\Concerns\Browsershot\CanBuildRequest;
use App\Services\Wiki\Concerns\Browsershot\CanSendGetRequest;
use App\Services\Wiki\Resources\AccessoryResource;
use App\Services\Wiki\Resources\ArmorResource;
use App\Services\Wiki\Resources\DynamicResource;
use App\Services\Wiki\Resources\WeaponResource;

class WikiService
{
    use CanBuildRequest;
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

    public function armor(): ArmorResource
    {
        return new ArmorResource(
            service: $this,
        );
    }

    public function accessory(): AccessoryResource
    {
        return new AccessoryResource(
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
