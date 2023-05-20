<?php

use App\Http\Controllers\Api\Weaponry\AodController;
use Illuminate\Support\Facades\Route;

Route::controller(AodController::class)
    ->name('aod.')
    ->prefix('aod')
    ->group(function () {
        Route::get('{region}/item/{itemId}/price', 'itemPrice')->name('item');
    });

require __DIR__.'/v1.php';
