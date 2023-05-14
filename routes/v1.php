<?php

use App\Http\Controllers\Api\V1\CategoryController;
use App\Http\Controllers\Api\V1\SpellController;
use App\Http\Controllers\Api\V1\WeaponController;
use App\Http\Controllers\Api\V1\WeaponStatController;
use Illuminate\Support\Facades\Route;

/*
|--------------------------------------------------------------------------
| V1 Routes
|--------------------------------------------------------------------------
|
| api routes for v1
|
*/

Route::controller(CategoryController::class)
    ->name('categories.')
    ->prefix('categories')
    ->group(function () {
        Route::get('/', 'index')->name('index');
    });

Route::controller(WeaponController::class)
    ->name('weapons.')
    ->prefix('weapons')
    ->group(function () {
        Route::get('/', 'index')->name('index');
    });

Route::controller(SpellController::class)
    ->name('spells.')
    ->prefix('spells')
    ->group(function () {
        Route::get('/weapon/{weaponId}', 'byWeaponId')->name('byWeaponId');
    });

Route::controller(WeaponStatController::class)
    ->name('weaponStats.')
    ->prefix('weapon-stats')
    ->group(function () {
        Route::get('/weapon/{weaponId}', 'byWeaponId')->name('byWeaponId');
    });
