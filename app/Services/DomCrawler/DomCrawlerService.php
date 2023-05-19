<?php

namespace App\Services\DomCrawler;

use App\Services\Concerns\BuildCrawler;
use App\Services\DomCrawler\Resources\ArmorResource;
use App\Services\DomCrawler\Resources\WeaponResource;

class DomCrawlerService
{
    use BuildCrawler;

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
}
