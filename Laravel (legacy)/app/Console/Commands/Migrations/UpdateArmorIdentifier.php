<?php

namespace App\Console\Commands\Migrations;

use App\Models\Armor;
use App\Services\AlbionOnlineData\AlbionOnlineDataService;
use Illuminate\Console\Command;

class UpdateArmorIdentifier extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'update:armor-identifier';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Update Armor Identifier';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $items = app(AlbionOnlineDataService::class)->items();
        $armors = Armor::query()
            ->get();
        foreach ($armors as $armor) {
            $item = $items->firstWhere('LocalizedNames.EN-US', $armor->name);
            $armor->update([
                'identifier' => $item['UniqueName'],
            ]);
        }
    }
}
