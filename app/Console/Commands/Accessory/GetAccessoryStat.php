<?php

namespace App\Console\Commands\Accessory;

use App\Actions\AccessoryStat\UpdateAccessoryStat;
use App\Models\Accessory;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetAccessoryStat extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:accessory-stat';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Accessory Stat From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $accessories = Accessory::query()
            ->where('path', '!=', null)
            ->whereNotIn('path', ['/wiki/Yule_Ram', '/wiki/Bronze_Battle_Rhino'])
            ->get();

        foreach ($accessories as $accessory) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($accessory->path))
                ->getBody()
                ->__toString();
            $data = app(DomCrawlerService::class)
                ->accessory()
                ->statList($html);
            foreach ($data as $item) {
                $quality = data_get($item, 'Item Quality');
                $tier = data_get($item, 'Tier');
                if ($quality && $tier) {
                    unset($item['Item Quality']);
                    unset($item['Tier']);
                    $stats = [];
                    foreach ($item as $key => $value) {
                        $stats[] = [
                            'name' => $key,
                            'value' => $value,
                        ];
                    }
                    app(UpdateAccessoryStat::class)
                        ->execute([
                            'accessory_id' => $accessory->id,
                            'quality' => $quality,
                            'enchantment' => Str::after($tier, '.') ?: 0,
                            'stats' => $stats,
                        ]);
                }
            }
        }
    }
}
