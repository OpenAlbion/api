<?php

namespace App\Console\Commands\Weapon;

use App\Actions\Spell\UpdateSpell;
use App\Actions\WeaponSpell\UpdateWeaponSpell;
use App\Models\Weapon;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetWeaponSpell extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:weapon-spell';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Weapon Spell From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $weapons = Weapon::query()
            ->where('subcategory_id', 69)
            ->where('path', '!=', null)
            ->get();

        foreach ($weapons as $weapon) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($weapon->path))
                ->toHtml();
            $data = app(DomCrawlerService::class)
                ->weapon()
                ->spellList($html);
            foreach ($data as $item) {
                $spell = app(UpdateSpell::class)
                    ->execute($item);
                app(UpdateWeaponSpell::class)
                    ->execute([
                        'weapon_id' => $weapon->id,
                        'spell_id' => $spell->id,
                    ]);
            }
        }
    }
}
