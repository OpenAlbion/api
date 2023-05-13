<?php

namespace App\Services\Concerns;

use Symfony\Component\DomCrawler\Crawler;

trait BuildCrawler
{
    public function buildCrawler(string $html): Crawler
    {
        return new Crawler($html);
    }
}
