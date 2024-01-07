<?php

namespace App\Console\Commands\Consumable;

use App\Actions\ConsumableCrafting\UpdateConsumableCrafting;
use App\Models\Category;
use App\Models\Consumable;
use App\Services\AlbionOnlineData\AlbionOnlineDataService;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Render\RenderService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetConsumableCrafting extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:consumable-crafting';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Consumable Crafting From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $aodItems = app(AlbionOnlineDataService::class)->items();

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
                ->toHtml();
            $data = app(DomCrawlerService::class)
                ->consumable()
                ->foodCraftingList($html);
            foreach ($data as $item) {
                $perCraft = data_get($item, 'Per Craft');
                $tier = data_get($item, 'Tier');
                if ($tier) {
                    unset($item['Per Craft']);
                    unset($item['Tier']);
                    $requirements = [];
                    foreach ($item as $key => $value) {
                        $aodItem = $aodItems->filter(function ($item) use ($key) {
                            if (isset($item['LocalizedNames']['EN-US'])) {
                                return strcasecmp($item['LocalizedNames']['EN-US'], $key) === 0;
                            }

                            return false;
                        })->first();
                        $name = optional($aodItem)['UniqueName'];
                        if ($value > 0) {
                            $requirements[] = [
                                'name' => $key,
                                'identifier' => $name,
                                'icon' => app(RenderService::class)->renderItem($name),
                                'value' => (int) $value,
                            ];
                        }
                    }
                    app(UpdateConsumableCrafting::class)
                        ->execute([
                            'consumable_id' => $consumable->id,
                            'per_craft' => $perCraft,
                            'enchantment' => Str::after($tier, '.') ?: 0,
                            'requirements' => $requirements,
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
                ->toHtml();
            $data = app(DomCrawlerService::class)
                ->consumable()
                ->potionCraftingList($html);
            foreach ($data as $item) {
                $perCraft = data_get($item, 'Per Craft');
                $tier = data_get($item, 'Tier');
                if ($tier) {
                    unset($item['Per Craft']);
                    unset($item['Tier']);
                    $requirements = [];
                    foreach ($item as $key => $value) {
                        $aodItem = $aodItems->filter(function ($item) use ($key) {
                            if (isset($item['LocalizedNames']['EN-US'])) {
                                return strcasecmp($item['LocalizedNames']['EN-US'], $key) === 0;
                            }

                            return false;
                        })->first();
                        $name = optional($aodItem)['UniqueName'];
                        if ($value > 0) {
                            $requirements[] = [
                                'name' => $key,
                                'identifier' => $name,
                                'icon' => app(RenderService::class)->renderItem($name),
                                'value' => (int) $value,
                            ];
                        }
                    }

                    app(UpdateConsumableCrafting::class)
                        ->execute([
                            'consumable_id' => $consumable->id,
                            'per_craft' => $perCraft,
                            'enchantment' => Str::after($tier, '.') ?: 0,
                            'requirements' => $requirements,
                        ]);
                }
            }
        }
    }
}
