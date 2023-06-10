<?php

namespace App\Providers;

use Illuminate\Cache\RateLimiting\Limit;
use Illuminate\Foundation\Support\Providers\RouteServiceProvider as ServiceProvider;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\RateLimiter;
use Illuminate\Support\Facades\Route;

class RouteServiceProvider extends ServiceProvider
{
    /**
     * The path to your application's "home" route.
     *
     * Typically, users are redirected here after authentication.
     *
     * @var string
     */
    public const HOME = '/dashboard';

    /**
     * Define your route model bindings, pattern filters, and other route configuration.
     */
    public function boot(): void
    {
        RateLimiter::for('api', function (Request $request) {
            $apiToken = $request->header('Authorization') ?: $request->query('api_token');

            return Limit::perMinute(60)->by($apiToken ?: $request->ip());
        });

        $this->routes(function () {
            Route::middleware('api')
                ->prefix('api')
                ->group(base_path('routes/api.php'));

            Route::middleware(['api', 'apiToken'])
                ->prefix('api/v1')
                ->group(base_path('routes/v1.php'));

            Route::middleware(['api', 'apiToken'])
                ->prefix('api/v2')
                ->name('v2.')
                ->group(base_path('routes/v2.php'));

            Route::middleware(['api', 'weaponryKey'])
                ->prefix('api/weaponry')
                ->name('weaponry.')
                ->group(base_path('routes/weaponry.php'));

            Route::middleware('web')
                ->group(base_path('routes/web.php'));
        });
    }
}
