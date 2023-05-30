<?php

namespace App\Console\Commands\Accessory;

use App\Actions\Accessory\UpdateAccessory;
use App\Models\Category;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;

class GetBag extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:bag';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Bag From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $html = app(WikiService::class)
            ->accessory()
            ->bagList()
            ->getBody()
            ->__toString();

        $data = app(DomCrawlerService::class)
            ->accessory()
            ->bagList($html);

        $category = Category::query()
            ->where('name', 'Bag')
            ->firstOrFail();

        $subcategory = Category::query()
            ->where('name', 'Normal Bags')
            ->firstOrFail();

        foreach ($data as $item) {
            app(UpdateAccessory::class)
                ->execute(array_merge($item, [
                    'category_id' => $category->id,
                    'subcategory_id' => $subcategory->id,
                ]));
        }
    }
}
