<?php

namespace App\Console\Commands\Armor;

use App\Actions\ArmorStat\UpdateArmorStat;
use App\Models\Armor;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetArmorStat extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:armor-stat';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Armor Stat From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $armors = Armor::query()
            ->where('path', '!=', null)
            ->get();

        foreach ($armors as $armor) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($armor->path))
                ->getBody()
                ->__toString();
            $data = app(DomCrawlerService::class)
                ->armor()
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
                    app(UpdateArmorStat::class)
                        ->execute([
                            'armor_id' => $armor->id,
                            'quality' => $quality,
                            'enchantment' => Str::after($tier, '.') ?: 0,
                            'stats' => $stats,
                        ]);
                }
            }
        }
    }
}
