<?php

use App\Http\Controllers\Api\Weaponry\AodController;
use App\Http\Controllers\Api\Weaponry\SearchController;
use Illuminate\Support\Facades\Route;

Route::controller(AodController::class)
    ->name('aod.')
    ->prefix('aod')
    ->group(function () {
        Route::get('{region}/item/{itemId}/price', 'itemPrice')->name('item');
    });

Route::controller(SearchController::class)
    ->group(function () {
        Route::get('search', 'search')->name('search');
    });

require __DIR__.'/v1.php';
