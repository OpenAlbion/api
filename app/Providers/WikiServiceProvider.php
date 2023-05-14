<?php

namespace App\Providers;

use App\Services\Wiki\WikiService;
use Illuminate\Http\Request;
use Illuminate\Support\ServiceProvider;
use Illuminate\Support\Str;

class WikiServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     */
    public function register(): void
    {
        //
    }

    /**
     * Bootstrap any application services.
     */
    public function boot(): void
    {
        $this->app->singleton(
            abstract: WikiService::class,
            concrete: fn () => new WikiService(
                baseUrl: config('services.wiki.base_url')
            ),
        );

        $this->registerMacros();
    }

    private function registerMacros()
    {
        Str::macro('wikiLink', function (string $str): string {
            return match ($str) {
                '/wiki/Hammer' => '/wiki/Hammers',
                '/wiki/Bow' => '/wiki/Bows',
                '/wiki/Spear' => '/wiki/Spears',
                default => $str
            };
        });

        Str::macro('wikiPath', function (?string $str): ?string {
            if ($str) {
                return urldecode($str);
            }

            return $str;
        });

        Str::macro('albionTire', function (string $str): ?int {
            if (Str::contains($str, 'Beginner\'s', true)) {
                return 1;
            } elseif (Str::contains($str, 'Novice\'s', true)) {
                return 2;
            } elseif (Str::contains($str, 'Journeyman\'s', true)) {
                return 3;
            } elseif (Str::contains($str, 'Adept\'s', true)) {
                return 4;
            } elseif (Str::contains($str, 'Expert\'s', true)) {
                return 5;
            } elseif (Str::contains($str, 'Grandmaster\'s', true)) {
                return 7;
            } elseif (Str::contains($str, 'Master\'s', true)) {
                return 6;
            } elseif (Str::contains($str, ['Elder\'s', 'Rosalia\'s'], true)) {
                return 8;
            }

            return null;
        });

        Str::macro('albionItemPower', function (?string $str): ?int {
            if ($str) {
                return (int) Str::replace(',', '', $str);
            }

            return null;
        });

        Str::macro('albionIdentifier', function (string $str): string {
            return pathinfo(basename($str), PATHINFO_FILENAME);
        });

        Str::macro('wikiPreview', function (string $str): string {
            return 'https://wiki.albiononline.com'.$str;
        });

        Request::macro('generateCacheKey', function () {
            $url = request()->url();
            $queryParams = request()->query();

            ksort($queryParams);

            $queryString = http_build_query($queryParams);

            $fullUrl = "{$url}?{$queryString}";

            return sha1($fullUrl);
        });
    }
}
