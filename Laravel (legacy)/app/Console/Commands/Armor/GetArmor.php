<?php

namespace App\Console\Commands\Armor;

use App\Actions\Armor\UpdateArmor;
use App\Enums\CategoryType;
use App\Models\Category;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetArmor extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:armor';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Armor From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $subcategories = Category::query()
            ->has('parent')
            ->with('parent')
            ->where('type', CategoryType::ARMOR)
            ->where('path', '!=', null)
            ->get();
        foreach ($subcategories as $subcategory) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($subcategory->path))
                ->toHtml();

            $data = app(DomCrawlerService::class)
                ->armor()
                ->list($html);
            foreach ($data as $item) {
                app(UpdateArmor::class)
                    ->execute(array_merge($item, [
                        'category_id' => $subcategory->parent->id,
                        'subcategory_id' => $subcategory->id,
                    ]));
            }
        }
    }
}
