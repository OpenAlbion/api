<?php

namespace App\Console\Commands\Consumable;

use App\Actions\Consumable\UpdateConsumable;
use App\Models\Category;
use Illuminate\Console\Command;

class GetPotion extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:potion';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Potion From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $data = [
            [
                'name' => 'Minor Energy Potion',
                'tier' => '2',
                'item_power' => '300',
                'path' => '/wiki/Minor_Energy_Potion',
                'category' => 'Potions',
                'subcategory' => 'Energy',
            ],
            [
                'name' => 'Minor Healing Potion',
                'tier' => '2',
                'item_power' => '300',
                'path' => '/wiki/Minor_Healing_Potion',
                'category' => 'Potions',
                'subcategory' => 'Healing',
            ],
            [
                'name' => 'Minor Gigantify Potion',
                'tier' => '3',
                'item_power' => '500',
                'path' => '/wiki/Minor_Gigantify_Potion',
                'category' => 'Potions',
                'subcategory' => 'Gigantify',
            ],
            [
                'name' => 'Minor Resistance Potion',
                'tier' => '3',
                'item_power' => '500',
                'path' => '/wiki/Minor_Resistance_Potion',
                'category' => 'Potions',
                'subcategory' => 'Resistance',
            ],
            [
                'name' => 'Minor Sticky Potion',
                'tier' => '3',
                'item_power' => '500',
                'path' => '/wiki/Minor_Sticky_Potion',
                'category' => 'Potions',
                'subcategory' => 'Sticky',
            ],
            [
                'name' => 'Energy Potion',
                'tier' => '4',
                'item_power' => '700',
                'path' => '/wiki/Energy_Potion',
                'category' => 'Potions',
                'subcategory' => 'Energy',
            ],
            [
                'name' => 'Healing Potion',
                'tier' => '4',
                'item_power' => '700',
                'path' => '/wiki/Healing_Potion',
                'category' => 'Potions',
                'subcategory' => 'Healing',
            ],
            [
                'name' => 'Minor Poison Potion',
                'tier' => '4',
                'item_power' => '700',
                'path' => '/wiki/Minor_Poison_Potion',
                'category' => 'Potions',
                'subcategory' => 'Poision',
            ],
            [
                'name' => 'Gigantify Potion',
                'tier' => '5.0',
                'item_power' => '800',
                'path' => '/wiki/Gigantify_Potion',
                'category' => 'Potions',
                'subcategory' => 'Gigantify',
            ],
            [
                'name' => 'Resistance Potion',
                'tier' => '5.0',
                'item_power' => '800',
                'path' => '/wiki/Resistance_Potion',
                'category' => 'Potions',
                'subcategory' => 'Resistance',
            ],
            [
                'name' => 'Sticky Potion',
                'tier' => '5.0',
                'item_power' => '800',
                'path' => '/wiki/Sticky_Potion',
                'category' => 'Potions',
                'subcategory' => 'Sticky',
            ],
            [
                'name' => 'Major Energy Potion',
                'tier' => '6.0',
                'item_power' => '900',
                'path' => '/wiki/Major_Energy_Potion',
                'category' => 'Potions',
                'subcategory' => 'Energy',
            ],
            [
                'name' => 'Major Healing Potion',
                'tier' => '6.0',
                'item_power' => '900',
                'path' => '/wiki/Major_Healing_Potion',
                'category' => 'Potions',
                'subcategory' => 'Healing',
            ],
            [
                'name' => 'Poison Potion',
                'tier' => '6.0',
                'item_power' => '900',
                'path' => '/wiki/Poison_Potion',
                'category' => 'Potions',
                'subcategory' => 'Poision',
            ],
            [
                'name' => 'Major Gigantify Potion',
                'tier' => '7.0',
                'item_power' => '980',
                'path' => '/wiki/Major_Gigantify_Potion',
                'category' => 'Potions',
                'subcategory' => 'Gigantify',
            ],
            [
                'name' => 'Major Resistance Potion',
                'tier' => '7.0',
                'item_power' => '980',
                'path' => '/wiki/Major_Resistance_Potion',
                'category' => 'Potions',
                'subcategory' => 'Resistance',
            ],
            [
                'name' => 'Major Sticky Potion',
                'tier' => '7.0',
                'item_power' => '980',
                'path' => '/wiki/Major_Sticky_Potion',
                'category' => 'Potions',
                'subcategory' => 'Sticky',
            ],
            [
                'name' => 'Focus Restoration Potion',
                'tier' => '8.0',
                'item_power' => '0',
                'path' => '/wiki/Focus_Restoration_Potion',
                'category' => 'Potions',
                'subcategory' => 'Focus',
            ],
            [
                'name' => 'Invisibility Potion',
                'tier' => '8.0',
                'item_power' => '1060',
                'path' => '/wiki/Invisibility_Potion',
                'category' => 'Potions',
                'subcategory' => 'Invisible',
            ],
            [
                'name' => 'Major Poison Potion',
                'tier' => '8.0',
                'item_power' => '1060',
                'path' => '/wiki/Major_Poison_Potion',
                'category' => 'Potions',
                'subcategory' => 'Poision',
            ],
        ];

        foreach ($data as $item) {
            $subcategory = Category::query()
                ->where('name', $item['subcategory'])
                ->first();
            app(UpdateConsumable::class)
                ->execute(array_merge($item, [
                    'category_id' => $subcategory->parent->id,
                    'subcategory_id' => $subcategory->id,
                ]));
        }
    }
}
