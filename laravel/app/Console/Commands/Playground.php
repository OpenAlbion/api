<?php

namespace App\Console\Commands;

use App\Services\Wiki\WikiService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class Playground extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'app:playground';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Playground';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $html = app(WikiService::class)
            ->dynamic()
            ->get(Str::wikiLink('/wiki/Adept\'s_Bloodmoon_Staff'))
            ->toHtml();
        dd($html);
    }
}
