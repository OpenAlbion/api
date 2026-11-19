<?php

namespace App\Console\Commands\Migrations;

use App\Models\Accessory;
use App\Models\Armor;
use App\Models\Weapon;
use Illuminate\Console\Command;

class UpdateItemPower extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'update:item-power';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Update Item Power';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $weapons = Weapon::query()
            ->where('item_power', null)
            ->has('weaponStats')
            ->withWhereHas('weaponStats', function ($query) {
                $query->where('quality', 'Normal')
                    ->where('enchantment', 0);
            })
            ->get();

        foreach ($weapons as $weapon) {
            $ip = collect($weapon->weaponStats->first()->stats)->where('name', 'Item Power')->first();
            if ($ip) {
                $weapon->update([
                    'item_power' => $ip['value'],
                ]);
            }
        }

        $armors = Armor::query()
            ->where('item_power', null)
            ->has('armorStats')
            ->withWhereHas('armorStats', function ($query) {
                $query->where('quality', 'Normal')
                    ->where('enchantment', 0);
            })
            ->get();

        foreach ($armors as $armor) {
            $ip = collect($armor->armorStats->first()->stats)->where('name', 'Item Power')->first();
            if ($ip) {
                $armor->update([
                    'item_power' => $ip['value'],
                ]);
            }
        }

        $accessories = Accessory::query()
            ->where('item_power', null)
            ->has('accessoryStats')
            ->withWhereHas('accessoryStats', function ($query) {
                $query->where('quality', 'Normal')
                    ->where('enchantment', 0);
            })
            ->get();

        foreach ($accessories as $accessory) {
            $ip = collect($accessory->accessoryStats->first()->stats)->where('name', 'Item Power')->first();
            if ($ip) {
                $accessory->update([
                    'item_power' => $ip['value'] ?: null,
                ]);
            }
        }
    }
}
