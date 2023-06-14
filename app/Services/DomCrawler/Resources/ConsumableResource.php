<?php

namespace App\Services\DomCrawler\Resources;

use App\Services\DomCrawler\DomCrawlerService;
use Illuminate\Support\Str;

class ConsumableResource
{
    public function __construct(
        private readonly DomCrawlerService $service,
    ) {
    }

    public function foodList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $foods = [];
        $foodContainer = $dom->filter('#mw-content-text > div > table > tbody > tr');

        foreach (range(1, $foodContainer->count() - 1) as $i) {
            $foods[] = [
                'name' => trim($foodContainer->eq($i)->filter('td')->eq(0)->text()),
                'icon' => null,
                'tier' => $foodContainer->eq($i)->filter('td')->eq(1)->text(),
                'item_power' => $foodContainer->eq($i)->filter('td')->eq(2)->text(),
                'path' => $foodContainer->eq($i)->filter('td')->eq(0)->filter('a')->attr('href'),
            ];
        }

        return $foods;
    }

    public function foodStatList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $stats = [];
        $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(12)');

        $isInvalidTable = function ($container) {
            return ! ($container->count());
        };

        $colCount = $statContainer->filter('tr')->eq(1)->filter('th')->count();

        $rowCount = $statContainer->filter('tr')->count();
        $stats = [];
        foreach (range(0, $colCount - 1) as $col) {
            $currentStats = [];
            foreach (range(2, $rowCount - 1) as $row) {
                $tier = $statContainer->filter('tr')->eq(1)->filter('th')->eq($col)->text();
                $header = $statContainer->filter('tr')->eq($row)->filter('td')->eq(0)->text();
                $currentStats[$tier]['Item Quality'] = 'Normal';
                $currentStats[$tier]['Tier'] = Str::replace('Tier ', '', $tier);
                $currentStats[$tier][$header] = $statContainer->filter('tr')->eq($row)->filter('td')->eq($col + 1)->text();
            }
            $stats[] = $currentStats[$tier];
        }

        return $stats;
    }

    public function potionStatList(string $html): array
    {
        $dom = $this->service->buildCrawler($html);

        $stats = [];
        $statContainer = $dom->filter('#mw-content-text > div > table:nth-child(10)');

        $isInvalidTable = function ($container) {
            return ! ($container->count());
        };

        $colCount = $statContainer->filter('tr')->eq(1)->filter('th')->count();

        $rowCount = $statContainer->filter('tr')->count();
        $stats = [];
        foreach (range(0, $colCount - 1) as $col) {
            $currentStats = [];
            foreach (range(2, $rowCount - 1) as $row) {
                $tier = $statContainer->filter('tr')->eq(1)->filter('th')->eq($col)->text();
                $header = $statContainer->filter('tr')->eq($row)->filter('td')->eq(0)->text();
                $currentStats[$tier]['Item Quality'] = 'Normal';
                $currentStats[$tier]['Tier'] = Str::replace('Tier ', '', $tier);
                $currentStats[$tier][$header] = $statContainer->filter('tr')->eq($row)->filter('td')->eq($col + 1)->text();
            }
            $stats[] = $currentStats[$tier];
        }

        return $stats;
    }

    public function info(string $html): string
    {
        $dom = $this->service->buildCrawler($html);
        $container = $dom->filter('#mw-content-text > div > p');
        $info = '';
        foreach(range(1, $container->count() - 2) as $i) {
            $info .= "{$container->eq($i)->text()}\n";
        }
        return $info;
    }
}
