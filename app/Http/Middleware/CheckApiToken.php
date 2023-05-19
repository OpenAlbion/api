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
        $token = $request->header('Authorization') ?: $request->query('api_token');
        if ($token) {
            $apiToken = cache()->remember(
                $request->apiTokenCacheKey(),
                config('settings.cache_seconds'),
                function () use ($token) {
                    return ApiToken::query()
                        ->where('token', $token)
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
