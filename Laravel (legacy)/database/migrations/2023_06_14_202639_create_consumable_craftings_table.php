<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('consumable_craftings', function (Blueprint $table) {
            $table->id();
            $table->foreignId('consumable_id');
            $table->integer('per_craft');
            $table->integer('enchantment');
            $table->jsonb('requirements');
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('consumable_craftings');
    }
};
