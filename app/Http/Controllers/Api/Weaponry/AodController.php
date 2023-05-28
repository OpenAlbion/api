<?php

namespace App\Http\Controllers\Api\Weaponry;

use App\Http\Controllers\Controller;
use Illuminate\Support\Facades\Http;

class AodController extends Controller
{
    public function itemPrice($region, $itemId)
    {
        // this is not public api, only use for app
        // fetch data form albion online data project and response to application, we do not modify response, we just cache data for 3 minutes to optimize api performance
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
