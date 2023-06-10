<?php

namespace App\Console\Commands\Consumable;

use App\Actions\Consumable\UpdateConsumable;
use App\Enums\CategoryType;
use App\Models\Category;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetFood extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:food';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Food From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $subcategories = Category::query()
            ->has('parent')
            ->with('parent')
            ->where('type', CategoryType::Consumable)
            ->where('path', '!=', null)
            ->get();
        foreach ($subcategories as $subcategory) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($subcategory->path))
                ->getBody()
                ->__toString();

            $data = app(DomCrawlerService::class)
                ->consumable()
                ->foodList($html);
            foreach ($data as $item) {
                app(UpdateConsumable::class)
                    ->execute(array_merge($item, [
                        'category_id' => $subcategory->parent->id,
                        'subcategory_id' => $subcategory->id,
                    ]));
            }
        }
    }
}
