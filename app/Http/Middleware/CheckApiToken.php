<?php

namespace App\Http\Middleware;

use App\Models\ApiToken;
use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class CheckApiToken
{
    /**
     * Handle an incoming request.
     *
     * @param  \Closure(\Illuminate\Http\Request): (\Symfony\Component\HttpFoundation\Response)  $next
     */
    public function handle(Request $request, Closure $next): Response
    {
        if ($request->query('api_token')) {
            $apiToken = cache()->remember(
                $request->apiTokenCacheKey(),
                config('settings.cache_seconds'),
                function () use ($request) {
                    return ApiToken::query()
                        ->where('token', $request->input('api_token'))
                        ->first();
                }
            );
            if ($apiToken instanceof ApiToken) {
                return $next($request);
            }
        }

        return response()->json([
            'message' => 'Invalid Api Token!',
        ], 401);
    }
}
