<?php

namespace App\Console\Commands\Migrations;

use App\Models\Accessory;
use App\Models\Category;
use Illuminate\Console\Command;

class UpdateCapeSubcategory extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'update:cape-subcategory';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Update Cape Subcategory';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $capeCatgory = Category::query()
            ->where('name', 'Cape')
            ->first();

        $normalCapeCatgory = Category::query()
            ->where('name', 'Normal Capes')
            ->first();

        $factionCapeCatgory = Category::query()
            ->where('name', 'Faction Capes')
            ->first();

        Accessory::query()
            ->whereHas('category', function ($query) use ($capeCatgory) {
                $query->where('id', $capeCatgory->id);
            })
            ->doesntHave('spells')
            ->update([
                'subcategory_id' => $normalCapeCatgory->id,
            ]);

        Accessory::query()
            ->whereHas('category', function ($query) use ($capeCatgory) {
                $query->where('id', $capeCatgory->id);
            })
            ->has('spells')
            ->update([
                'subcategory_id' => $factionCapeCatgory->id,
            ]);
    }
}
