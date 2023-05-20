<?php

namespace App\Services\DomCrawler\Resources;

use App\Enums\SpellSlot;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Support\Str;

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

    public function spellList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $spells = [];

        $spellContainer = $dom->filter('#mw-content-text > div > p');
        if ($spellContainer->count()) {
            foreach (range(1, 2) as $i) {
                $slotContainer = $spellContainer->eq($i)->filter('a');
                if (Str::contains($spellContainer->eq($i)->text(), 'D spell slot')) {
                    $slot = SpellSlot::D;
                } elseif (Str::contains($spellContainer->eq($i)->text(), 'R spell slot')) {
                    $slot = SpellSlot::R;
                } elseif (Str::contains($spellContainer->eq($i)->text(), 'F spell slot')) {
                    $slot = SpellSlot::F;
                } elseif (Str::contains($spellContainer->eq($i)->text(), 'passive slot')) {
                    $slot = SpellSlot::PASSIVE;
                }
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
