<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class() extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('weapon_stats', function (Blueprint $table) {
            $table->id();
            $table->foreignId('weapon_id');
            $table->string('quality');
            $table->integer('enchantment');
            $table->jsonb('stats');
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('weapon_stats');
    }
};
