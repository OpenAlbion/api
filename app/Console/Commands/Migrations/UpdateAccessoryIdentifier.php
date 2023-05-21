<?php

namespace App\Console\Commands\Migrations;

use App\Models\Accessory;
use App\Services\AlbionOnlineData\AlbionOnlineDataService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

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
            ->where('identifier', null)
            ->get();
        foreach ($accessories as $accessory) {
            $item = $items->filter(function ($item) use ($accessory) {
                if (isset($item['LocalizedNames']['EN-US'])) {
                    return strcasecmp($item['LocalizedNames']['EN-US'], Str::replace(' (Mount)', '', $accessory->name)) === 0;
                }
                return false;
            })->first();
            $name = optional($item)['UniqueName'];
            if ($name == null) {
                $name = Str::replace('@ITEMS_', '', $item['LocalizationNameVariable']);
            }
            $accessory->update([
                'identifier' => Str::before($item['UniqueName'], '@'),
            ]);
        }
    }
}
