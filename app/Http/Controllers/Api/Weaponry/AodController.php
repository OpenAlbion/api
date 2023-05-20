<?php

namespace App\Http\Controllers\Api\Weaponry;

use App\Http\Controllers\Controller;
use Illuminate\Support\Facades\Http;

class AodController extends Controller
{
    public function itemPrice($region, $itemId)
    {
        $response = cache()->remember(
            request()->generateCacheKey(),
            config('settings.cache_seconds'),
            function () use ($region, $itemId) {
                return Http::get("http://{$region}.albion-online-data.com/api/v2/stats/prices/{$itemId}.json", request()->all())->json();
            }
        );

        return response()->json($response);
    }
}
