<?php

namespace App\Console\Commands\Consumable;

use App\Models\Consumable;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetConsumableInfo extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:consumable-info';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Consumable Infro From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $consumables = Consumable::query()
            ->where('path', '!=', null)
            ->get();

        foreach ($consumables as $consumable) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($consumable->path))
                ->toHtml();
            $info = app(DomCrawlerService::class)
                ->consumable()
                ->info($html);
            $consumable->update([
                'info' => $info,
            ]);
        }
    }
}
