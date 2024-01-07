<?php

namespace App\Console\Commands\Weapon;

use App\Actions\Weapon\UpdateWeapon;
use App\Models\Category;
use Illuminate\Console\Command;

class GetShapeshifterStaff extends Command
{
    /**
     * The name and signature of the console command.
     *
     * @var string
     */
    protected $signature = 'get:shapeshifter-staff';

    /**
     * The console command description.
     *
     * @var string
     */
    protected $description = 'Get Shapeshifter Staff';

    /**
     * Execute the console command.
     */
    public function handle()
    {
        $subcategory = Category::with('parent')
            ->where('name', 'Shapeshifter Staff')
            ->first();

        $data = [
            [
                'name' => "Journeyman's Prowling Staff",
                'identifier' => null,
                'tier' => 3,
                'item_power' => 500,
                'path' => "/wiki/Journeyman's_Prowling_Staff",
            ],
            [
                'name' => "Adept's Prowling Staff",
                'identifier' => null,
                'tier' => 4,
                'item_power' => 700,
                'path' => "/wiki/Adept's_Prowling_Staff",
            ],
            [
                'name' => "Expert's Prowling Staff",
                'identifier' => null,
                'tier' => 5,
                'item_power' => 800,
                'path' => "/wiki/Expert's_Prowling_Staff",
            ],
            [
                'name' => "Master's Prowling Staff",
                'identifier' => null,
                'tier' => 6,
                'item_power' => 900,
                'path' => "/wiki/Master's_Prowling_Staff",
            ],
            [
                'name' => "Grandmaster's Prowling Staff",
                'identifier' => null,
                'tier' => 7,
                'item_power' => 1000,
                'path' => "/wiki/Grandmaster's_Prowling_Staff",
            ],
            [
                'name' => "Elder's Prowling Staff",
                'identifier' => null,
                'tier' => 8,
                'item_power' => 1100,
                'path' => "/wiki/Elder's_Prowling_Staff",
            ],

            [
                'name' => "Adept's Rootbound Staff",
                'identifier' => null,
                'tier' => 4,
                'item_power' => 700,
                'path' => "/wiki/Adept's_Rootbound_Staff",
            ],
            [
                'name' => "Expert's Rootbound Staff",
                'identifier' => null,
                'tier' => 5,
                'item_power' => 800,
                'path' => "/wiki/Expert's_Rootbound_Staff",
            ],
            [
                'name' => "Master's Rootbound Staff",
                'identifier' => null,
                'tier' => 6,
                'item_power' => 900,
                'path' => "/wiki/Master's_Rootbound_Staff",
            ],
            [
                'name' => "Grandmaster's Rootbound Staff",
                'identifier' => null,
                'tier' => 7,
                'item_power' => 1000,
                'path' => "/wiki/Grandmaster's_Rootbound_Staff",
            ],
            [
                'name' => "Elder's Rootbound Staff",
                'identifier' => null,
                'tier' => 8,
                'item_power' => 1100,
                'path' => "/wiki/Elder's_Rootbound_Staff",
            ],

            [
                'name' => "Adept's Primal Staff",
                'identifier' => null,
                'tier' => 4,
                'item_power' => 700,
                'path' => "/wiki/Adept's_Primal_Staff",
            ],
            [
                'name' => "Expert's Primal Staff",
                'identifier' => null,
                'tier' => 5,
                'item_power' => 800,
                'path' => "/wiki/Expert's_Primal_Staff",
            ],
            [
                'name' => "Master's Primal Staff",
                'identifier' => null,
                'tier' => 6,
                'item_power' => 900,
                'path' => "/wiki/Master's_Primal_Staff",
            ],
            [
                'name' => "Grandmaster's Primal Staff",
                'identifier' => null,
                'tier' => 7,
                'item_power' => 1000,
                'path' => "/wiki/Grandmaster's_Primal_Staff",
            ],
            [
                'name' => "Elder's Primal Staff",
                'identifier' => null,
                'tier' => 8,
                'item_power' => 1100,
                'path' => "/wiki/Elder's_Primal_Staff",
            ],

            [
                'name' => "Adept's Bloodmoon Staff",
                'identifier' => null,
                'tier' => 4,
                'item_power' => 725,
                'path' => "/wiki/Adept's_Bloodmoon_Staff",
            ],
            [
                'name' => "Expert's Bloodmoon Staff",
                'identifier' => null,
                'tier' => 5,
                'item_power' => 825,
                'path' => "/wiki/Expert's_Bloodmoon_Staff",
            ],
            [
                'name' => "Master's Bloodmoon Staff",
                'identifier' => null,
                'tier' => 6,
                'item_power' => 925,
                'path' => "/wiki/Master's_Bloodmoon_Staff",
            ],
            [
                'name' => "Grandmaster's Bloodmoon Staff",
                'identifier' => null,
                'tier' => 7,
                'item_power' => 1025,
                'path' => "/wiki/Grandmaster's_Bloodmoon_Staff",
            ],
            [
                'name' => "Elder's Bloodmoon Staff",
                'identifier' => null,
                'tier' => 8,
                'item_power' => 1125,
                'path' => "/wiki/Elder's_Bloodmoon_Staff",
            ],

            [
                'name' => "Adept's Hellspawn Staff",
                'identifier' => null,
                'tier' => 4,
                'item_power' => 750,
                'path' => "/wiki/Adept's_Hellspawn_Staff",
            ],
            [
                'name' => "Expert's Hellspawn Staff",
                'identifier' => null,
                'tier' => 5,
                'item_power' => 850,
                'path' => "/wiki/Expert's_Hellspawn_Staff",
            ],
            [
                'name' => "Master's Hellspawn Staff",
                'identifier' => null,
                'tier' => 6,
                'item_power' => 950,
                'path' => "/wiki/Master's_Hellspawn_Staff",
            ],
            [
                'name' => "Grandmaster's Hellspawn Staff",
                'identifier' => null,
                'tier' => 7,
                'item_power' => 1050,
                'path' => "/wiki/Grandmaster's_Hellspawn_Staff",
            ],
            [
                'name' => "Elder's Hellspawn Staff",
                'identifier' => null,
                'tier' => 8,
                'item_power' => 1150,
                'path' => "/wiki/Elder's_Hellspawn_Staff",
            ],

            [
                'name' => "Adept's Earthrune Staff",
                'identifier' => null,
                'tier' => 4,
                'item_power' => 775,
                'path' => "/wiki/Adept's_Earthrune_Staff",
            ],
            [
                'name' => "Expert's Earthrune Staff",
                'identifier' => null,
                'tier' => 5,
                'item_power' => 875,
                'path' => "/wiki/Expert's_Earthrune_Staff",
            ],
            [
                'name' => "Master's Earthrune Staff",
                'identifier' => null,
                'tier' => 6,
                'item_power' => 975,
                'path' => "/wiki/Master's_Earthrune_Staff",
            ],
            [
                'name' => "Grandmaster's Earthrune Staff",
                'identifier' => null,
                'tier' => 7,
                'item_power' => 1075,
                'path' => "/wiki/Grandmaster's_Earthrune_Staff",
            ],
            [
                'name' => "Elder's Earthrune Staff",
                'identifier' => null,
                'tier' => 8,
                'item_power' => 1175,
                'path' => "/wiki/Elder's_Earthrune_Staff",
            ],

            [
                'name' => "Adept's Lightcaller",
                'identifier' => null,
                'tier' => 4,
                'item_power' => 800,
                'path' => "/wiki/Adept's_Lightcaller",
            ],
            [
                'name' => "Expert's Lightcaller",
                'identifier' => null,
                'tier' => 5,
                'item_power' => 900,
                'path' => "/wiki/Expert's_Lightcaller",
            ],
            [
                'name' => "Master's Lightcaller",
                'identifier' => null,
                'tier' => 6,
                'item_power' => 1000,
                'path' => "/wiki/Master's_Lightcaller",
            ],
            [
                'name' => "Grandmaster's Lightcaller",
                'identifier' => null,
                'tier' => 7,
                'item_power' => 1100,
                'path' => "/wiki/Grandmaster's_Lightcaller",
            ],
            [
                'name' => "Elder's Lightcaller",
                'identifier' => null,
                'tier' => 8,
                'item_power' => 1200,
                'path' => "/wiki/Elder's_Lightcaller",
            ],
        ];

        foreach ($data as $item) {
            app(UpdateWeapon::class)
                ->execute(array_merge($item, [
                    'category_id' => $subcategory->parent->id,
                    'subcategory_id' => $subcategory->id,
                    'icon' => '',
                ]));
        }

    }
}
