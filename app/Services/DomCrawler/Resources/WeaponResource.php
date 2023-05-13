<?php

namespace App\Services\DomCrawler\Resources;

use App\Enums\SpellSlot;
use App\Models\Spell;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Render\RenderService;
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
                    'tire' => $weaponContainer->eq($i)->filter('td')->eq(2)->text(),
                    'item_power' => $weaponContainer->eq($i)->filter('td')->eq(3)->text(),
                    'path' => $weaponContainer->eq($i)->filter('td')->eq(1)->filter('a')->attr('href'),
                ];
            }
        } else {
            $weaponContainer = $dom->filter('#mw-content-text > div > table > tbody > tr');
            if ($weaponContainer->count()) {
                foreach (range(1, $weaponContainer->count() - 1) as $i) {
                    $name = $weaponContainer->eq($i)->filter('td')->eq(0)->text();
                    if ($weaponContainer->eq($i)->filter('td')->eq(0)->filter('a')->count()) { // for all weapons
                        $path = $weaponContainer->eq($i)->filter('td')->eq(0)->filter('a')->attr('href');
                    } else { // for only tome of spells
                        $path = '/wiki/'.Str::replace(' ', '_', $name);
                    }
                    $weapons[] = [
                        'name' => $name,
                        'icon' => $weaponContainer->eq($i)->filter('td')->eq(0)->filter('img')->attr('src'),
                        'tire' => Str::albionTire($name),
                        'item_power' => null,
                        'path' => $path,
                    ];
                }
            }
        }

        return $weapons;
    }

    public function spellList(string $html, ?string $ref = null): array
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
                        $existSpell = Spell::query()
                            ->where('ref', Str::wikiLink($slotContainer->eq($ii)->attr('href')))
                            ->first();
                        if (! $existSpell) {
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
                        } else {
                            $spells[] = [
                                'name' => $existSpell->name,
                                'icon' => app(RenderService::class)->renderSpell($existSpell->identifier),
                                'slot' => $existSpell->slot,
                                'attributes' => $existSpell->attributes,
                                'description' => $existSpell->description,
                                'preview' => $existSpell->preview,
                                'ref' => $existSpell->ref,
                            ];
                        }
                    }
                }
            }
        }

        return $spells;
    }
}
