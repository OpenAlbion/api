<?php

namespace App\Console\Commands\Weapon;

use App\Actions\Category\UpdateCategory;
use App\Enums\CategoryType;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;

class GetWeaponCategory extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:weapon-category';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Weapon Category From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $html = app(WikiService::class)
            ->weapon()
            ->categoryList()
            ->getBody()
            ->__toString();

        $data = app(DomCrawlerService::class)
            ->weapon()
            ->categoryList($html);

        foreach ($data as $item) {
            $category = app(UpdateCategory::class)
                ->execute([
                    'name' => data_get($item, 'primary'),
                    'type' => CategoryType::WEAPON,
                ]);
            app(UpdateCategory::class)
                ->execute([
                    'name' => data_get($item, 'secondary'),
                    'parent_id' => $category->id,
                    'path' => data_get($item, 'path'),
                    'type' => CategoryType::WEAPON,
                ]);
        }
    }
}
