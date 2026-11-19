<?php

namespace App\Console\Commands\Accessory;

use App\Actions\AccessorySpell\UpdateAccessorySpell;
use App\Actions\Spell\UpdateSpell;
use App\Models\Accessory;
use App\Models\Category;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class GetAccessorySpell extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:accessory-spell';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Accessory Spell From Albion Wiki';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $mountCategory = Category::query()
            ->where('name', 'Mount')
            ->first();

        $acessories = Accessory::query()
            ->where('path', '!=', null)
            ->where('category_id', '!=', $mountCategory->id)
            ->get();

        foreach ($acessories as $accessory) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($accessory->path))
                ->toHtml();
            $data = app(DomCrawlerService::class)
                ->accessory()
                ->spellList($html);
            foreach ($data as $item) {
                $spell = app(UpdateSpell::class)
                    ->execute($item);
                app(UpdateAccessorySpell::class)
                    ->execute([
                        'accessory_id' => $accessory->id,
                        'spell_id' => $spell->id,
                    ]);
            }
        }
    }
}
