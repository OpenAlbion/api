<?php

namespace App\Console\Commands\Migrations;

use App\Models\Spell;
use App\Services\DomCrawler\DomCrawlerService;
use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class UpdateSpellDescription extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'update:spell-description';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Update Spell Description';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $spells = Spell::query()
            ->where('description', 'Sanitize Spell Description')
            ->get();
        foreach ($spells as $spell) {
            $html = app(WikiService::class)
                ->dynamic()
                ->get(Str::wikiLink($spell->ref))
                ->getBody()
                ->__toString();

            $data = app(DomCrawlerService::class)
                ->spell()
                ->detail($html);
            $spell->update([
                'description' => Str::sanitizeSpellDescription($data['description']),
            ]);
        }
    }
}
