<?php

namespace App\Console\Commands\Weapon;

use App\Actions\Category\UpdateCategory;
use App\Enums\CategoryType;
use App\Models\Category;
use Illuminate\Console\Command;

class GetShapeshifterStaffCategory extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:shapeshifter-staff-category';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Shapeshifter Staff Category';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $category = Category::where('name', 'Hunter Weapons')
            ->where('parent_id', null)
            ->first();

        app(UpdateCategory::class)
            ->execute([
                'name' => 'Shapeshifter Staff',
                'parent_id' => $category->id,
                'path' => null,
                'type' => CategoryType::WEAPON,
            ]);
    }
}
