<?php

namespace App\Services\DomCrawler\Resources;

use App\Services\DomCrawler\DomCrawlerService;
use Illuminate\Support\Str;

class SpellResource
{
    public function __construct(
        private readonly DomCrawlerService $service,
    ) {
    }

    public function detail(string $html): array
    {
        $dom = $this->service->buildCrawler($html);
        $attributes = [];
        $attributeTable = $dom->filter('#mw-content-text > div > table:nth-child(4) > tbody > tr');
        if (! $attributeTable->count()) {
            $attributeTable = $dom->filter('#mw-content-text > div > table:nth-child(5) > tbody > tr');
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

        $preview = $dom->filter('#mw-content-text > div > p:nth-child(9) > a > img');

        return [
            'name' => $attributeTable->eq(0)->filter('td:nth-child(1) > span > a')->text(),
            'icon' => $attributeTable->eq(0)->filter('td:nth-child(1) > span > a > img')->attr('src'),
            'attributes' => $attributes,
            'description' => $attributeTable->eq(0)->filter('td:nth-child(4)')->html(),
            'preview' => $preview->count()
                ? Str::wikiPreview($preview->attr('src'))
                : null,
        ];
    }
}
