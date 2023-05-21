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

    public function mountSpellList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $spells = [];

        $spellContainer = $dom->filter('#mw-content-text > div > ul > li');

        foreach (range(0, $spellContainer->count() -1) as $s) {
            $spell = $spellContainer->eq($s);
            if (Str::contains($spell->text(), 'unique abilites', true) || Str::contains($spell->text(), 'Unique abilities', true)) {
                $abilities = $spell->filter('a');
                $slot = SpellSlot::MOUNT;
                if ($abilities->count()) {
                    foreach (range(0, $abilities->count() - 1) as $i) {
                        if (Str::contains(Str::wikiLink($abilities->eq($i)->attr('href')), '/wiki/')) {
                            $spellHtml = app(WikiService::class)
                        ->dynamic()
                        ->get(Str::wikiLink($abilities->eq($i)->attr('href')))
                        ->getBody()
                        ->__toString();
                            $spellDom = $this->service->buildCrawler($spellHtml);
                            $attributes = [];
                            $attributeTable = $spellDom->filter('#mw-content-text > div > table > tbody > tr');
                            if ($attributeTable->eq(1)->filter('img')->count()) {
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
                            } else {
                                $attributeTable = $spellDom->filter('#mw-content-text > div > table:nth-child(4) > tbody > tr');
                                if (! $attributeTable->count()) {
                                    $attributeTable = $spellDom->filter('#mw-content-text > div > table:nth-child(5) > tbody > tr');
                                }
                                if ($attributeTable->count()) {
                                    foreach (range(0, $attributeTable->count() - 2) as $attributeIndex) {
                                        $nameIndex = 1;
                                        $valueIndex = 2;
                                        if ($attributeIndex > 0) {
                                            $nameIndex = 0;
                                            $valueIndex = 1;
                                        }
                                        if ($attributeTable->eq($attributeIndex)->filter('td')->eq($nameIndex)->count()) {
                                            $attributes[] = [
                                                'name' => $attributeTable->eq($attributeIndex)->filter('td')->eq($nameIndex)->text(),
                                                'value' => $attributeTable->eq($attributeIndex)->filter('td')->eq($valueIndex)->text(),
                                            ];
                                        }
                                    }
                                }
                                if ($attributeTable->eq(0)->filter('td:nth-child(1) > span > a')->count()) {
                                    $spells[] = [
                                        'name' => $attributeTable->eq(0)->filter('td:nth-child(1) > span > a')->text(),
                                        'icon' => $attributeTable->eq(0)->filter('td:nth-child(1) > span > a > img')->attr('src'),
                                        'slot' => $slot,
                                        'attributes' => $attributes,
                                        'description' => $attributeTable->eq(0)->filter('td:nth-child(4)')->html(),
                                        'preview' => null,
                                        'ref' => Str::wikiLink($spell->filter('a')->attr('href')),
                                    ];
                                }
                            }
                        }
                    }
                }
            }
        }


        return $spells;
    }

    public function statList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $stats = [];
        $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(14)');

        $isInvalidTable = function ($container) {
            return ! ($container->count() && Str::contains($container->text(), 'Mount Hit Points'));
        };

        if ($isInvalidTable($statContainer)) {
            $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(18)');
        }

        if ($isInvalidTable($statContainer)) {
            $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(15)');
        }

        if ($isInvalidTable($statContainer)) {
            $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(10)');
        }

        if ($isInvalidTable($statContainer)) {
            $statContainer = $dom->filter('#mw-content-text > div > table.wikitable.sortable.jquery-tablesorter');
        }

        if ($isInvalidTable($statContainer)) {
            $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(6)');
        };

        // must be last
        if ($isInvalidTable($statContainer)) {
            $statContainer = $dom->filter('#mw-content-text > div > table.wikitable.sortable');
        }

        $headers = [];
        $headerContainer = $statContainer->filter('th');
        $forceNormalQuality = false;
        if ($headerContainer->eq(1)->text() != 'Item Quality') {
            $headers[] = 'Item Quality';
            $forceNormalQuality = true;
        }
        foreach (range(1, $headerContainer->count() - 1) as $i) {
            $headers[] = $headerContainer->eq($i)->text();
        }
        $bodyContainer = $statContainer->filter('tr');
        foreach (range(2, $bodyContainer->count() - 1) as $i) {
            $rowContainer = $bodyContainer->eq($i);
            $details = [];
            if ($forceNormalQuality) {
                $details['Item Quality'] = 'Normal';
            }
            foreach (range(0, $rowContainer->filter('td')->count() - 1) as $ii) {
                if ($forceNormalQuality) {
                    $ii = $ii + 1;
                }
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

    public function mountList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $mounts = [];
        $mountContainer = $dom->filter('#mw-content-text > div > table:nth-child(11) > tbody > tr');
        foreach (range(0, $mountContainer->count() - 1) as $i) {
            $category = $mountContainer->eq($i)->filter('td')->eq(0)->text();
            $itemContainer = $mountContainer->eq($i)->filter('td')->eq(1)->filter('span');
            foreach (range(0, $itemContainer->count() -1) as $ii) {
                $name = pathinfo(basename($itemContainer->eq($ii)->filter('img')->attr('src')), PATHINFO_FILENAME);
                $mounts[] = [
                    'category' => $category,
                    'name' => $name,
                    // 'icon' => $itemContainer->eq($ii)->filter('image')->attr('src'),
                    'icon' => null,
                    'tier' => null,
                    'item_power' => null,
                    'path' => $itemContainer->eq($ii)->filter('a')->attr('href')
                ];
            }
        }

        return $mounts;
    }
}
