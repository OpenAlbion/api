<?php

namespace App\Console\Commands\Weapon;

use App\Actions\WeaponStat\UpdateWeaponStat;
use App\Models\Weapon;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetWeaponStat extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:weapon-stat';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Weapon Stat From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $weapons = Weapon::query()
            ->where('path', '!=', null)
            ->whereNotIn('path', [
                '/wiki/Journeyman\'s_Nature_Staff', // skip error page
            ])
            ->get();

        foreach ($weapons as $weapon) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($weapon->path))
                ->getBody()
                ->__toString();
            $data = app(DomCrawlerService::class)
                ->weapon()
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
                    app(UpdateWeaponStat::class)
                        ->execute([
                            'weapon_id' => $weapon->id,
                            'quality' => $quality,
                            'enchantment' => Str::after($tier, '.') ?: 0,
                            'stats' => $stats,
                        ]);
                }
            }
        }
    }
}
