<?php

namespace App\Console\Commands\Consumable;

use App\Actions\Category\UpdateCategory;
use App\Enums\CategoryType;
use Illuminate\Console\Command;

class GetConsumableCategory extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:consumable-category';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Consumable Category From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $data = [
            [
                'primary' => 'Foods',
                'secondary' => 'Omelette',
                'path' => '/wiki/Omelette',
            ],
            [
                'primary' => 'Foods',
                'secondary' => 'Pie',
                'path' => '/wiki/Pie',
            ],
            [
                'primary' => 'Foods',
                'secondary' => 'Salad',
                'path' => '/wiki/Salad',
            ],
            [
                'primary' => 'Foods',
                'secondary' => 'Sandwich',
                'path' => '/wiki/Sandwich',
            ],
            [
                'primary' => 'Foods',
                'secondary' => 'Soup',
                'path' => '/wiki/Soup',
            ],
            [
                'primary' => 'Foods',
                'secondary' => 'Stew',
                'path' => '/wiki/Stew',
            ],
            [
                'primary' => 'Foods',
                'secondary' => 'Roast',
                'path' => '/wiki/Roast',
            ],
            [
                'primary' => 'Potions',
                'secondary' => 'Energy',
                'path' => null,
            ],
            [
                'primary' => 'Potions',
                'secondary' => 'Healing',
                'path' => null,
            ],
            [
                'primary' => 'Potions',
                'secondary' => 'Gigantify',
                'path' => null,
            ],
            [
                'primary' => 'Potions',
                'secondary' => 'Resistance',
                'path' => null,
            ],
            [
                'primary' => 'Potions',
                'secondary' => 'Sticky',
                'path' => null,
            ],
            [
                'primary' => 'Potions',
                'secondary' => 'Poision',
                'path' => null,
            ],
            [
                'primary' => 'Potions',
                'secondary' => 'Invisible',
                'path' => null,
            ],
            [
                'primary' => 'Potions',
                'secondary' => 'Focus',
                'path' => null,
            ],
        ];

        foreach ($data as $item) {
            $category = app(UpdateCategory::class)
                ->execute([
                    'name' => data_get($item, 'primary'),
                    'type' => CategoryType::Consumable,
                    'path' => data_get($item, 'path'),
                    'version' => 2,
                ]);
            app(UpdateCategory::class)
                ->execute([
                    'name' => data_get($item, 'secondary'),
                    'parent_id' => $category->id,
                    'path' => data_get($item, 'path'),
                    'type' => CategoryType::Consumable,
                    'version' => 2,
                ]);
        }
    }
}
