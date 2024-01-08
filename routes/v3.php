<?php

use App\Http\Controllers\Api\V3\AccessoryController;
use App\Http\Controllers\Api\V3\AccessoryStatController;
use App\Http\Controllers\Api\V3\ArmorController;
use App\Http\Controllers\Api\V3\ArmorStatController;
use App\Http\Controllers\Api\V3\CategoryController;
use App\Http\Controllers\Api\V3\ConsumableController;
use App\Http\Controllers\Api\V3\ConsumableCraftingController;
use App\Http\Controllers\Api\V3\ConsumableStatController;
use App\Http\Controllers\Api\V3\SpellController;
use App\Http\Controllers\Api\V3\WeaponController;
use App\Http\Controllers\Api\V3\WeaponStatController;
use Illuminate\Support\Facades\Route;

/*
|--------------------------------------------------------------------------
| V3 Routes
|--------------------------------------------------------------------------
|
| api routes for v3
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
        Route::get('/armor/{armorId}', 'byArmorId')->name('byArmorId');
        Route::get('/accessory/{accessoryId}', 'byAccessoryId')->name('byAccessoryId');
    });

Route::controller(WeaponStatController::class)
    ->name('weaponStats.')
    ->prefix('weapon-stats')
    ->group(function () {
        Route::get('/weapon/{weaponId}', 'byWeaponId')->name('byWeaponId');
    });

Route::controller(ArmorStatController::class)
    ->name('armorStats.')
    ->prefix('armor-stats')
    ->group(function () {
        Route::get('/armor/{armorId}', 'byArmorId')->name('byArmorId');
    });

Route::controller(ArmorController::class)
    ->name('armors.')
    ->prefix('armors')
    ->group(function () {
        Route::get('/', 'index')->name('index');
    });

Route::controller(AccessoryController::class)
    ->name('accessories.')
    ->prefix('accessories')
    ->group(function () {
        Route::get('/', 'index')->name('index');
    });

Route::controller(AccessoryStatController::class)
    ->name('accessoryStats.')
    ->prefix('accessory-stats')
    ->group(function () {
        Route::get('/accessory/{accessoryId}', 'byAccessoryId')->name('byAccessoryId');
    });

Route::controller(ConsumableController::class)
    ->name('consumables.')
    ->prefix('consumables')
    ->group(function () {
        Route::get('/', 'index')->name('index');
    });

Route::controller(ConsumableStatController::class)
    ->name('consumableStats.')
    ->prefix('consumable-stats')
    ->group(function () {
        Route::get('/consumable/{consumableId}', 'byConsumableId')->name('byConsumableId');
    });

Route::controller(ConsumableCraftingController::class)
    ->name('consumableCraftings.')
    ->prefix('consumable-craftings')
    ->group(function () {
        Route::get('/consumable/{consumableId}', 'byConsumableId')->name('byConsumableId');
    });
