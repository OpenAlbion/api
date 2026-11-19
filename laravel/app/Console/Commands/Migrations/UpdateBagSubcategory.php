<?php

namespace App\Console\Commands\Migrations;

use App\Models\Accessory;
use App\Models\Category;
use Illuminate\Console\Command;

class UpdateBagSubcategory extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'update:bag-subcategory';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Update Bag Subcategory';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $bagCategory = Category::query()
            ->where('name', 'Bag')
            ->first();

        $normalBagCatgory = Category::query()
            ->where('name', 'Normal Bags')
            ->first();

        Accessory::query()
            ->whereHas('category', function ($query) use ($bagCategory) {
                $query->where('id', $bagCategory->id);
            })
            ->update([
                'subcategory_id' => $normalBagCatgory->id,
            ]);
    }
}
