<?php

namespace App\Services\DomCrawler\Resources;

use App\Services\DomCrawler\DomCrawlerService;

class ArmorResource
{
    public function __construct(
        private readonly DomCrawlerService $service,
    ) {
    }

    public function list(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $armors = [];
        $armorContainer = $dom->filter('#mw-content-text > div > table:nth-child(5) > tbody > tr');

        $isInvalidTable = function ($container) {
            return ! $container->count();
        };

        if ($isInvalidTable($armorContainer)) {
            $armorContainer = $dom->filter('#mw-content-text > div > table:nth-child(13) > tbody > tr');
        }

        if ($isInvalidTable($armorContainer)) {
            $armorContainer = $dom->filter('#mw-content-text > div > table > tbody > tr');
        }

        foreach (range(1, $armorContainer->count() - 1) as $i) {
            $armors[] = [
                'name' => $armorContainer->eq($i)->filter('td')->eq(0)->text(),
                'icon' => null,
                'tier' => $armorContainer->eq($i)->filter('td')->eq(1)->text(),
                'item_power' => $armorContainer->eq($i)->filter('td')->eq(2)->text(),
                'path' => $armorContainer->eq($i)->filter('td')->eq(0)->filter('a')->attr('href'),
            ];
        }

        return $armors;
    }
}
