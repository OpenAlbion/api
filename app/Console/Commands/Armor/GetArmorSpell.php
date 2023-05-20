<?php

namespace App\Console\Commands\Armor;

use App\Actions\Spell\UpdateSpell;
use App\Models\Armor;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetArmorSpell extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:armor-spell';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Armor Spell From Albion Wiki';

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
                ->spellList($html);
            foreach ($data as $item) {
                $spell = app(UpdateSpell::class)
                    ->execute($item);
                // app(UpdateWeaponSpell::class)
                //     ->execute([
                //         'weapon_id' => $armor->id,
                //         'spell_id' => $spell->id,
                //     ]);
            }
        }
    }
}
