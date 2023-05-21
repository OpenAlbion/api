<?php

namespace App\Console\Commands\Accessory;

use App\Actions\Accessory\UpdateAccessory;
use App\Models\Category;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;

class GetCape extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:cape';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Cape From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $html = app(WikiService::class)
            ->accessory()
            ->capeList()
            ->getBody()
            ->__toString();

        $data = app(DomCrawlerService::class)
            ->accessory()
            ->capeList($html);

        $category = Category::query()
            ->where('name', 'Cape')
            ->firstOrFail();

        foreach ($data as $item) {
            app(UpdateAccessory::class)
                ->execute(array_merge($item, [
                    'category_id' => $category->id,
                ]));
        }
    }
}
