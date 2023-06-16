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

    public function download()
    {
        return redirect()->away('https://play.google.com/store/apps/details?id=com.openalbion.weaponry&fbclid=IwAR0yP3o-qjj5b64V8T5ZxB51zUR9PKqSn4Vf7PQ1XrwC4PjR-PdkPHrEep4');
    }
}
