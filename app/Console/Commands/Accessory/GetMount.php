<?php

namespace App\Console\Commands\Accessory;

use App\Actions\Accessory\UpdateAccessory;
use App\Actions\Category\UpdateCategory;
use App\Enums\CategoryType;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetMount extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:mount';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Mount From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $html = app(WikiService::class)
            ->accessory()
            ->mountList()
            ->getBody()
            ->__toString();

        $data = app(DomCrawlerService::class)
            ->accessory()
            ->mountList($html);

        $category = app(UpdateCategory::class)
            ->execute([
                'name' => 'Mount',
                'type' => CategoryType::ACCESSORY,
                'path' => '/wiki/Cloth_Robes',
            ]);
        foreach ($data as $item) {
            $subcategory = app(UpdateCategory::class)
                ->execute([
                    'name' => Str::replace(':', '', data_get($item, 'category')),
                    'parent_id' => $category->id,
                    'type' => CategoryType::ACCESSORY,
                ]);
            app(UpdateAccessory::class)
                ->execute(array_merge($item, [
                    'category_id' => $category->id,
                    'subcategory_id' => $subcategory->id,
                ]));
        }
    }
}
