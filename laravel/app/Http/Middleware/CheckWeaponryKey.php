<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class CheckWeaponryKey
{
    /**
     * Handle an incoming request.
     *
     * @param  \Closure(\Illuminate\Http\Request): (\Symfony\Component\HttpFoundation\Response)  $next
     */
    public function handle(Request $request, Closure $next): Response
    {
        $request->merge([
            'weaponry' => false,
        ]);
        $weaponryToken = config('settings.weaponry_key');
        if ($weaponryToken && $weaponryToken === $request->header('X-WEAPONRY-KEY')) {
            $request->merge([
                'weaponry' => true,
            ]);

            return $next($request);
        }

        return response()->json([
            'message' => 'Invalid Api Token!',
        ], 401);
    }
}
