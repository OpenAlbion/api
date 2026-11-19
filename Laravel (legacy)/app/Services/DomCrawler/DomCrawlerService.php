<?php

namespace App\Services\DomCrawler;

use App\Services\DomCrawler\Concerns\CanBuildCrawler;
use App\Services\DomCrawler\Resources\AccessoryResource;
use App\Services\DomCrawler\Resources\ArmorResource;
use App\Services\DomCrawler\Resources\ConsumableResource;
use App\Services\DomCrawler\Resources\SpellResource;
use App\Services\DomCrawler\Resources\WeaponResource;

class DomCrawlerService
{
    use CanBuildCrawler;

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

    public function consumable(): ConsumableResource
    {
        return new ConsumableResource(
            service: $this,
        );
    }

    public function spell(): SpellResource
    {
        return new SpellResource(
            service: $this,
        );
    }
}
