<?php

namespace App\Http\Controllers\Api\Weaponry;

use App\Http\Controllers\Controller;

class ApplicationController extends Controller
{
    public function versionCheck()
    {
        return response()->json([
            'data' => [
                'title' => 'Update Available!',
                'description' => 'Removed firebase app check.',
                'version' => '1.0.3',
                'force' => true,
            ],
        ]);
    }
}
