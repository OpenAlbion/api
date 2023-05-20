<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Kreait\Firebase\Exception\AppCheck\FailedToVerifyAppCheckToken;
use Symfony\Component\HttpFoundation\Response;

class FirebaseAppCheck
{
    /**
     * Handle an incoming request.
     *
     * @param  \Closure(\Illuminate\Http\Request): (\Symfony\Component\HttpFoundation\Response)  $next
     */
    public function handle(Request $request, Closure $next): Response
    {
        $token = $request->header('X-Firebase-AppCheck');
        if ($token) {
            $appCheck = app('firebase.app_check');
            try {
                $result = $appCheck->verifyToken($token);
                if ($result->token->aud[1] == config('settings.firebase_project_id')) {
                    $request->merge([
                        'app_check' => true,
                    ]);

                    return $next($request);
                }
            } catch (FailedToVerifyAppCheckToken $e) {
            }
        }
        $request->merge([
            'app_check' => false,
        ]);

        return response()->json([
            'message' => 'Invalid App Check',
        ]);
    }
}
