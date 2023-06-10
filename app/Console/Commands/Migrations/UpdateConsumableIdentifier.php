<?php

namespace App\Console\Commands\Migrations;

use App\Models\Consumable;
use App\Services\AlbionOnlineData\AlbionOnlineDataService;
use Illuminate\Console\Command;

class UpdateConsumableIdentifier extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'update:consumable-identifier';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Update Consumable Identifier';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $items = app(AlbionOnlineDataService::class)->items();
        $consumables = Consumable::query()
            ->get();
        foreach ($consumables as $consumable) {
            $item = $items->firstWhere('LocalizedNames.EN-US', $consumable->name);
            $consumable->update([
                'identifier' => $item['UniqueName'],
            ]);
        }
    }
}
