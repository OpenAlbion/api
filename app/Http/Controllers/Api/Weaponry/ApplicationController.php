<?php

namespace App\Http\Controllers\Api\Weaponry;

use App\Http\Controllers\Controller;

class ApplicationController extends Controller
{
    public function versionCheck()
    {
        return response()->json([
            'title' => 'Update Available!',
            'description' => 'Added: Social Account',
            'version' => '1.0.5',
            'force' => false,
        ]);
    }
}
