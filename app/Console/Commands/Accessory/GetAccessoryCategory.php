<?php

namespace App\Console\Commands\Accessory;

use App\Actions\Category\UpdateCategory;
use App\Enums\CategoryType;
use Illuminate\Console\Command;

class GetAccessoryCategory extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:accessory-category';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Accessory Category From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $data = [
            [
                'primary' => 'Cape',
                'secondary' => 'Normal Capes',
                'path' => '/wiki/Cape',
            ],
            [
                'primary' => 'Cape',
                'secondary' => 'Faction Capes',
                'path' => '/wiki/Cape',
            ],
            [
                'primary' => 'Bag',
                'secondary' => 'Normal Bags',
                'path' => '/wiki/Bag',
            ],
        ];

        foreach ($data as $item) {
            $category = app(UpdateCategory::class)
                ->execute([
                    'name' => data_get($item, 'primary'),
                    'type' => CategoryType::ACCESSORY,
                    'path' => data_get($item, 'path'),
                ]);
            app(UpdateCategory::class)
                ->execute([
                    'name' => data_get($item, 'secondary'),
                    'parent_id' => $category->id,
                    'path' => data_get($item, 'path'),
                    'type' => CategoryType::ACCESSORY,
                ]);
        }
    }
}
