<?php

namespace App\Console\Commands\Migrations;

use App\Actions\Weapon\UpdateWeapon;
use App\Models\Category;
use App\Models\Weapon;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class CreateMissingOffHandItem extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'create:missing-off-hand-item';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Create missing Off-Hand items';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        // $data = [
        //     '/wiki/Adept%27s_Leering_Cane',
        //     '/wiki/Adept%27s_Cryptcandle',
        //     '/wiki/Adept%27s_Sacred_Scepter',
        // ];
        // foreach($data as $item) {
        //     $html = app(WikiService::class)
        //         ->dynamic()
        //         ->get(Str::wikiLink($item))
        //         ->getBody()
        //         ->__toString();

        //     $data = app(DomCrawlerService::class)
        //         ->weapon()
        //         ->offHandlist($html);
        //     foreach ($data as $item) {
        //         app(UpdateWeapon::class)
        //             ->execute(array_merge($item, [
        //                 'category_id' => 9,
        //                 'subcategory_id' => 15,
        //             ]));
        //     }
        // }

        $data = [
            '/wiki/Adept%27s_Eye_of_Secrets',
            '/wiki/Adept%27s_Muisak',
            '/wiki/Adept%27s_Taproot',
            '/wiki/Adept%27s_Celestial_Censer',
        ];
        foreach($data as $item) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($item))
                ->getBody()
                ->__toString();

            $data = app(DomCrawlerService::class)
                ->weapon()
                ->offHandlist($html);
            foreach ($data as $item) {
                app(UpdateWeapon::class)
                    ->execute(array_merge($item, [
                        'category_id' => 16,
                        'subcategory_id' => 22,
                    ]));
            }
        }
    }
}
