<?php

namespace App\Console\Commands\Migrations;

use App\Models\Accessory;
use App\Services\AlbionOnlineData\AlbionOnlineDataService;
use Illuminate\Console\Command;

class UpdateAccessoryIdentifier extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'update:accessory-identifier';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Update Accessory Identifier';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $items = app(AlbionOnlineDataService::class)->items();
        $accessories = Accessory::query()
            ->get();
        foreach ($accessories as $accessory) {
            $item = $items->firstWhere('LocalizedNames.EN-US', $accessory->name);
            $accessory->update([
                'identifier' => $item['UniqueName'],
            ]);
        }
    }
}
