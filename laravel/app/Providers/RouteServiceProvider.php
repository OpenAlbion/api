<?php

namespace App\Providers;

use Exception;
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
            return Limit::perMinute(60)->by($request->ip());
        });

        Request::macro('generateCacheKey', function (): string {
            $url = request()->url();
            $queryParams = request()->query();

            if (request()->query('api_token')) {
                unset($queryParams['api_token']);
            }

            ksort($queryParams);

            $queryString = http_build_query($queryParams);

            $fullUrl = "{$url}?{$queryString}";

            return sha1($fullUrl);
        });

        Request::macro('apiTokenCacheKey', function (): string {
            $apiToken = request()->header('Authorization') ?: request()->query('api_token');
            if ($apiToken) {
                return sha1($apiToken);
            }
            throw new Exception('Invalid Api Token!');
        });

        $this->routes(function () {
            Route::middleware('api')
                ->prefix('api')
                ->group(base_path('routes/api.php'));

            Route::middleware(['api', 'apiToken', 'logRequest'])
                ->prefix('api/v1')
                ->group(base_path('routes/v1.php'));

            Route::middleware(['api', 'apiToken', 'logRequest'])
                ->prefix('api/v2')
                ->name('v2.')
                ->group(base_path('routes/v2.php'));

            Route::middleware(['api', 'logRequest'])
                ->prefix('api/v3')
                ->name('v3.')
                ->group(base_path('routes/v3.php'));

            Route::middleware(['api', 'weaponryKey'])
                ->prefix('api/weaponry')
                ->name('weaponry.')
                ->group(base_path('routes/weaponry.php'));

            Route::middleware(['api', 'weaponryKey'])
                ->prefix('api/weaponryV2')
                ->name('weaponryV2.')
                ->group(base_path('routes/weaponryV2.php'));

            Route::middleware('web')
                ->group(base_path('routes/web.php'));
        });
    }
}
