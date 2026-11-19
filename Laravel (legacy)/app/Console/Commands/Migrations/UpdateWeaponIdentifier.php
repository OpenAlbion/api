<?php

namespace App\Console\Commands\Migrations;

use App\Models\Weapon;
use App\Services\AlbionOnlineData\AlbionOnlineDataService;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class UpdateWeaponIdentifier extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'update:weapon-identifier';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Update Weapon Identifier';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $items = app(AlbionOnlineDataService::class)->items();
        $weapons = Weapon::query()
            ->where('identifier', '')
            ->orWhere('identifier', null)
            ->get();
        foreach ($weapons as $weapon) {
            $item = $items->filter(function ($item) use ($weapon) {
                if (isset($item['LocalizedNames']['EN-US'])) {
                    return strcasecmp($item['LocalizedNames']['EN-US'], Str::replace(' (Mount)', '', $weapon->name)) === 0;
                }

                return false;
            })->first();
            $name = optional($item)['UniqueName'];
            if ($name == null) {
                $name = Str::replace('@ITEMS_', '', $item['LocalizationNameVariable']);
            }
            $weapon->update([
                'identifier' => Str::before($item['UniqueName'], '@'),
            ]);
        }
    }
}
