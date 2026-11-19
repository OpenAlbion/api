<?php

use App\Http\Controllers\Api\Weaponry\AodController;
use App\Http\Controllers\Api\Weaponry\ApplicationController;
use App\Http\Controllers\Api\Weaponry\BugReportController;
use App\Http\Controllers\Api\Weaponry\SearchController;
use Illuminate\Support\Facades\Route;

Route::controller(AodController::class)
    ->name('aod.')
    ->prefix('aod')
    ->group(function () {
        Route::get('{region}/item/{itemId}/price', 'itemPrice')->name('itemPrice');
    });

Route::controller(SearchController::class)
    ->group(function () {
        Route::get('search', 'v2Search')->name('search');
    });

Route::controller(BugReportController::class)
    ->group(function () {
        Route::post('bug-report', 'report')->name('bugReport');
    });

Route::controller(ApplicationController::class)
    ->group(function () {
        Route::get('version-check', 'versionCheck')->name('versionCheck');
    });

require __DIR__.'/v2.php';
