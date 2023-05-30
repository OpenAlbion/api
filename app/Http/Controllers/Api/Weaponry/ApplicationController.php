<?php

namespace App\Http\Controllers\Api\Weaponry;

use App\Http\Controllers\Controller;

class ApplicationController extends Controller
{
    public function versionCheck()
    {
        return response()->json([
            'title' => 'Update Available!',
            'description' => 'Improve: UI',
            'version' => '1.0.4',
            'force' => true,
        ]);
    }
}
