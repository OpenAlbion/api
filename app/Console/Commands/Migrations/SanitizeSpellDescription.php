<?php

namespace App\Console\Commands\Migrations;

use App\Models\Spell;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class SanitizeSpellDescription extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'sanitize:spell-description';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Sanitize Spell Description';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $data = Spell::query()
            ->get();
        foreach ($data as $item) {
            $item->update([
                'description' => Str::sanitizeSpellDescription($this->description)
            ]);
        }
    }
}
