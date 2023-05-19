<?php

namespace App\Console\Commands\Armor;

use App\Actions\Category\UpdateCategory;
use App\Enums\CategoryType;
use Illuminate\Console\Command;

class GetArmorCategory extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:armor-category';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Armor Category From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $data = [
            [
                'primary' => 'Cloth Armor',
                'secondary' => 'Robes',
                'path' => '/wiki/Cloth_Robes',
            ],
            [
                'primary' => 'Cloth Armor',
                'secondary' => 'Cowl',
                'path' => '/wiki/Cloth_Cowls',
            ],
            [
                'primary' => 'Cloth Armor',
                'secondary' => 'Sandals',
                'path' => '/wiki/Cloth_Sandals',
            ],
            [
                'primary' => 'Leather Armor',
                'secondary' => 'Jackets',
                'path' => '/wiki/Leather_Jackets',
            ],
            [
                'primary' => 'Leather Armor',
                'secondary' => 'Hoods',
                'path' => '/wiki/Leather_Hoods',
            ],
            [
                'primary' => 'Leather Armor',
                'secondary' => 'Shoes',
                'path' => '/wiki/Leather_Shoes',
            ],
            [
                'primary' => 'Plate Armor',
                'secondary' => 'Armors',
                'path' => '/wiki/Plate_Armors',
            ],
            [
                'primary' => 'Plate Armor',
                'secondary' => 'Helmets',
                'path' => '/wiki/Plate_Helmets',
            ],
            [
                'primary' => 'Plate Armor',
                'secondary' => 'Boots',
                'path' => '/wiki/Plate_Boots',
            ],
        ];

        foreach ($data as $item) {
            $category = app(UpdateCategory::class)
                ->execute([
                    'name' => data_get($item, 'primary'),
                    'type' => CategoryType::ARMOR,
                ]);
            app(UpdateCategory::class)
                ->execute([
                    'name' => data_get($item, 'secondary'),
                    'parent_id' => $category->id,
                    'path' => data_get($item, 'path'),
                    'type' => CategoryType::ARMOR,
                ]);
        }
    }
}
