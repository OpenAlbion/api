<?php

namespace App\Services\DomCrawler\Resources;

use App\Enums\SpellSlot;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Support\Str;

class AccessoryResource
{
    public function __construct(
        private readonly DomCrawlerService $service,
    ) {
    }

    public function capeList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $capes = [];
        $capeContainer = $dom->filter('#mw-content-text > div > table:nth-child(12) > tbody > tr');

        foreach (range(1, $capeContainer->count() - 1) as $i) {
            $capes[] = [
                'name' => $capeContainer->eq($i)->filter('td')->eq(0)->text(),
                'icon' => null,
                'tier' => $capeContainer->eq($i)->filter('td')->eq(1)->text(),
                'item_power' => $capeContainer->eq($i)->filter('td')->eq(2)->text(),
                'path' => $capeContainer->eq($i)->filter('td')->eq(0)->filter('a')->attr('href'),
            ];
        }

        return $capes;
    }

    public function bagList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $bags = [];
        $bagContainer = $dom->filter('#mw-content-text > div > table > tbody > tr:nth-child(1) > td:nth-child(2) > span');

        foreach (range(1, $bagContainer->count() - 1) as $i) {
            $bags[] = [
                'name' => $bagContainer->eq($i)->filter('a')->attr('title'),
                'icon' => null,
                'tier' => Str::albionTier($bagContainer->eq($i)->filter('a')->attr('title')),
                'item_power' => null,
                'path' => $bagContainer->eq($i)->filter('a')->attr('href'),
            ];
        }

        return $bags;
    }

    public function spellList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $spells = [];

        $spell = $dom->filter('#mw-content-text > div > p:nth-child(5)');
        if (Str::contains($spell->text(), 'unique spell')) {
            $slot = SpellSlot::PASSIVE;
            $spellHtml = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($spell->filter('a')->attr('href')))
                ->getBody()
                ->__toString();
            $spellDom = $this->service->buildCrawler($spellHtml);
            $attributes = [];
            $attributeTable = $spellDom->filter('#mw-content-text > div > table > tbody > tr');
            foreach (range(2, $attributeTable->count() - 1) as $i) {
                $attributes[] = [
                    'name' => $attributeTable->eq($i)->filter('td')->eq(0)->text(),
                    'value' => $attributeTable->eq($i)->filter('td')->eq(1)->text(),
                ];
            }

            $icon = $attributeTable->eq(1)->filter('img')->attr('src');
            $name = $attributeTable->eq(0)->text();
            $spells[] = [
                'name' => $name,
                'icon' => $icon,
                'slot' => $slot,
                'attributes' => $attributes,
                'description' => $spellDom->filter('#mw-content-text > div > p:nth-child(1)')->html(),
                'preview' => null,
                'ref' => Str::wikiLink($spell->filter('a')->attr('href')),
            ];
        }

        return $spells;
    }

    public function statList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $stats = [];
        $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(14)');

        $isInvalidTable = function ($container) {
            return ! ($container->count() && Str::contains($container->text(), 'Item Quality'));
        };

        if ($isInvalidTable) {
            $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(18)');
        }

        if ($isInvalidTable) {
            $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(15)');
        }

        if ($isInvalidTable) {
            $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(10)');
        }

        if ($isInvalidTable) {
            $statContainer = $dom->filter('#mw-content-text > div > table.wikitable.sortable.jquery-tablesorter');
        }

        // must be last
        if ($isInvalidTable) {
            $statContainer = $dom->filter('#mw-content-text > div > table.wikitable.sortable');
        }

        $headers = [];
        $headerContainer = $statContainer->filter('th');
        foreach (range(1, $headerContainer->count() - 1) as $i) {
            $headers[] = $headerContainer->eq($i)->text();
        }
        $bodyContainer = $statContainer->filter('tr');
        foreach (range(2, $bodyContainer->count() - 1) as $i) {
            $rowContainer = $bodyContainer->eq($i);
            $details = [];
            foreach (range(0, $rowContainer->filter('td')->count() - 1) as $ii) {
                if ($rowContainer->filter('td')->eq($ii)->count()) {
                    $details[$headers[$ii]] = $rowContainer->filter('td')->eq($ii)->text();
                }
            }
            if ($details && ! Str::contains($details['Item Quality'], 'Version')) {
                $stats[] = $details;
            }
        }

        return $stats;
    }
}
