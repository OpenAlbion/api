<?php

namespace App\Console\Commands\Migrations;

use App\Models\Accessory;
use App\Models\Category;
use Illuminate\Console\Command;
use Illuminate\Support\Str;

class UpdateMountTier extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'update:mount-tier';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Update Mount Tier';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $mountCategory = Category::query()
            ->where('name', 'Mount')
            ->first();

        $accessories = Accessory::query()
            ->where('category_id', $mountCategory->id)
            ->where('tier', null)
            ->get();
        foreach ($accessories as $accessory) {
            if (Str::startsWith($accessory->identifier, 'T2_')) {
                $accessory->update([
                    'tier' => 2
                ]);
            } elseif (Str::startsWith($accessory->identifier, 'T3_')) {
                $accessory->update([
                    'tier' => 3
                ]);
            } elseif (Str::startsWith($accessory->identifier, 'T4_')) {
                $accessory->update([
                    'tier' => 4
                ]);
            } elseif (Str::startsWith($accessory->identifier, 'T5_')) {
                $accessory->update([
                    'tier' => 5
                ]);
            } elseif (Str::startsWith($accessory->identifier, 'T6_')) {
                $accessory->update([
                    'tier' => 6
                ]);
            } elseif (Str::startsWith($accessory->identifier, 'T7_')) {
                $accessory->update([
                    'tier' => 7
                ]);
            } elseif (Str::startsWith($accessory->identifier, 'T8_')) {
                $accessory->update([
                    'tier' => 8
                ]);
            }
        }
    }
}
