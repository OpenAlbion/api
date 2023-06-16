<?php

namespace App\Http\Controllers\Api\Weaponry;

use App\Http\Controllers\Controller;

class ApplicationController extends Controller
{
    public function versionCheck()
    {
        return response()->json([
            'title' => 'Update Available!',
            'description' => "[+] Add Food & Potions\n[+] Add Crafting Process",
            'version' => '1.0.7',
            'force' => false,
        ]);
    }
}
