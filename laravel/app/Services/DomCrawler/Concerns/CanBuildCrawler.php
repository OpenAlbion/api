<?php

namespace App\Services\DomCrawler\Concerns;

use Symfony\Component\DomCrawler\Crawler;

trait CanBuildCrawler
{
    public function buildCrawler(string $html): Crawler
    {
        return new Crawler($html);
    }
}
