<?php

namespace App\Services\DomCrawler\Resources;

use App\Enums\SpellSlot;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Support\Str;

class WeaponResource
{
    public function __construct(
        private readonly DomCrawlerService $service,
    ) {
    }

    public function categoryList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $categories = [];

        $categoryContainer = $dom->filter('#mw-content-text > div > div > div');
        if ($categoryContainer->count()) {
            foreach (range(0, $categoryContainer->count() - 1) as $i) {
                $subcategoryContainer = $categoryContainer->eq($i)->filter('ul > li');
                foreach (range(0, $subcategoryContainer->count() - 1) as $ii) {
                    $categories[] = [
                        'primary' => $categoryContainer->eq($i)->filter('h3 > span')->text(),
                        'secondary' => $subcategoryContainer->eq($ii)->filter('a')->text(),
                        'path' => $subcategoryContainer->eq($ii)->filter('a')->attr('href'),
                    ];
                }
            }
        }

        return $categories;
    }

    public function list(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $weapons = [];

        $weaponContainer = $dom->filter('#mw-content-text > div > div > div:nth-child(1) > table > tbody > tr');
        if ($weaponContainer->count()) { // for all weapons
            foreach (range(1, $weaponContainer->count() - 1) as $i) {
                $weapons[] = [
                    'name' => $weaponContainer->eq($i)->filter('td')->eq(1)->text(),
                    'icon' => $weaponContainer->eq($i)->filter('td')->eq(0)->filter('img')->attr('src'),
                    'tier' => $weaponContainer->eq($i)->filter('td')->eq(2)->text(),
                    'item_power' => $weaponContainer->eq($i)->filter('td')->eq(3)->text(),
                    'path' => $weaponContainer->eq($i)->filter('td')->eq(1)->filter('a')->attr('href'),
                ];
            }
        } else {
            $weaponContainer = $dom->filter('#mw-content-text > div > table > tbody > tr');
            foreach (range(1, $weaponContainer->count() - 1) as $i) {
                $name = $weaponContainer->eq($i)->filter('td')->eq(0)->text();
                if ($weaponContainer->eq($i)->filter('td')->eq(0)->filter('a')->count()) { // for all weapons
                    $path = $weaponContainer->eq($i)->filter('td')->eq(0)->filter('a')->attr('href');
                } else { // for tome of spells
                    $path = '/wiki/'.Str::replace(' ', '_', $name);
                }
                $weapons[] = [
                    'name' => $name,
                    'icon' => $weaponContainer->eq($i)->filter('td')->eq(0)->filter('img')->attr('src'),
                    'tier' => Str::albionTier($name),
                    'item_power' => null,
                    'path' => $path,
                ];
            }
        }

        return $weapons;
    }

    public function offHandlist(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $weapons = [];

        $weaponContainer = $dom->filter('#mw-content-text > div > table:nth-child(19) > tbody > tr')->eq(1)->filter('td');
        if ($weaponContainer->count()) {
            foreach (range(0, $weaponContainer->count() - 1) as $i) {
                $img = $weaponContainer->eq($i)->filter('img')->attr('src');
                $weapons[] = [
                    'name' => Str::albionIdentifier($img),
                    'icon' => '',
                    'tier' => Str::albionTier(Str::albionIdentifier($img)),
                    'item_power' => '',
                    'path' => $weaponContainer->eq($i)->filter('a')->attr('href'),
                ];
            }
        }

        return $weapons;
    }

    public function spellList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $spells = [];

        $spellContainer = $dom->filter('#mw-content-text > div > p');
        if ($spellContainer->count()) {
            foreach (range(1, 4) as $i) {
                $slotContainer = $spellContainer->eq($i)->filter('a');
                $slot = match ($i) {
                    1 => SpellSlot::Q,
                    2 => SpellSlot::W,
                    3 => SpellSlot::E,
                    4 => SpellSlot::PASSIVE
                };
                foreach (range(0, $slotContainer->count() - 1) as $ii) {
                    if ($slotContainer->eq($ii)->count()) {
                        $spellHtml = app(WikiService::class)
                            ->dynamic()
                            ->get(Str::wikiLink($slotContainer->eq($ii)->attr('href')))
                            ->getBody()
                            ->__toString();
                        $spellDom = $this->service->buildCrawler($spellHtml);
                        $attributes = [];
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

                        $preview = $spellDom->filter('#mw-content-text > div > p:nth-child(9) > a > img');
                        if ($attributeTable->eq(0)->filter('td:nth-child(1) > span > a')->count()) {
                            $spells[] = [
                                'name' => $attributeTable->eq(0)->filter('td:nth-child(1) > span > a')->text(),
                                'icon' => $attributeTable->eq(0)->filter('td:nth-child(1) > span > a > img')->attr('src'),
                                'slot' => $slot,
                                'attributes' => $attributes,
                                'description' => $attributeTable->eq(0)->filter('td:nth-child(4)')->html(),
                                'preview' => $preview->count()
                                    ? Str::wikiPreview($preview->attr('src'))
                                    : null,
                                'ref' => Str::wikiLink($slotContainer->eq($ii)->attr('href')),
                            ];
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
            return ! ($container->count() && Str::contains($container->text(), 'Item Quality'));
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

        // must be last
        if ($isInvalidTable($statContainer)) {
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
