<?php

namespace App\Console\Commands\Consumable;

use App\Actions\ConsumableStat\UpdateConsumableStat;
use App\Models\Category;
use App\Models\Consumable;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetConsumableStat extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:consumable-stat';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Consumable Stat From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $foodCategory = Category::query()
            ->where('name', 'Foods')
            ->first();
        $potionCategory = Category::query()
            ->where('name', 'Potions')
            ->first();

        $consumables = Consumable::query()
            ->where('path', '!=', null)
            ->where('category_id', $foodCategory->id)
            ->get();

        foreach ($consumables as $consumable) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($consumable->path))
                ->getBody()
                ->__toString();
            $data = app(DomCrawlerService::class)
                ->consumable()
                ->foodStatList($html);
            foreach ($data as $item) {
                $quality = data_get($item, 'Item Quality');
                $tier = data_get($item, 'Tier');
                if ($quality && $tier) {
                    unset($item['Item Quality']);
                    unset($item['Tier']);
                    $stats = [];
                    foreach ($item as $key => $value) {
                        $stats[] = [
                            'name' => $key,
                            'value' => $value,
                        ];
                    }
                    app(UpdateConsumableStat::class)
                        ->execute([
                            'consumable_id' => $consumable->id,
                            'quality' => $quality,
                            'enchantment' => Str::after($tier, '.') ?: 0,
                            'stats' => $stats,
                        ]);
                }
            }
        }

        $consumables = Consumable::query()
            ->where('path', '!=', null)
            ->where('category_id', $potionCategory->id)
            ->whereNotIn('name', [
                'Focus Restoration Potion',
            ])
            ->get();

        foreach ($consumables as $consumable) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($consumable->path))
                ->getBody()
                ->__toString();
            $data = app(DomCrawlerService::class)
                ->consumable()
                ->potionStatList($html);
            foreach ($data as $item) {
                $quality = data_get($item, 'Item Quality');
                $tier = data_get($item, 'Tier');
                if ($quality && $tier) {
                    unset($item['Item Quality']);
                    unset($item['Tier']);
                    $stats = [];
                    foreach ($item as $key => $value) {
                        $stats[] = [
                            'name' => $key,
                            'value' => $value,
                        ];
                    }
                    app(UpdateConsumableStat::class)
                        ->execute([
                            'consumable_id' => $consumable->id,
                            'quality' => $quality,
                            'enchantment' => Str::after($tier, '.') ?: 0,
                            'stats' => $stats,
                        ]);
                }
            }
        }

        $focusRestorationPotion = Consumable::query()
            ->where('name', 'Focus Restoration Potion')
            ->first();
        app(UpdateConsumableStat::class)
            ->execute([
                'consumable_id' => $focusRestorationPotion->id,
                'quality' => 'Normal',
                'enchantment' => 0,
                'stats' => [
                    [
                        'name' => 'Item Power',
                        'value' => '0',
                    ],
                    [
                        'name' => 'Weight',
                        'value' => '0.1 kg',
                    ],
                    [
                        'name' => 'Restore Focus',
                        'value' => '+10000',
                    ],
                ],
            ]);
    }
}
