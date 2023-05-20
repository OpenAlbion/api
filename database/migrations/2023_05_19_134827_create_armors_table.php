<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class () extends Migration {
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::create('armors', function (Blueprint $table) {
            $table->id();
            $table->foreignId('category_id');
            $table->foreignId('subcategory_id');
            $table->string('name');
            $table->string('identifier')->nullable();
            $table->decimal('tier', 2, 1);
            $table->integer('item_power')->nullable();
            $table->string('path')->nullable();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     */
    public function down(): void
    {
        Schema::dropIfExists('armors');
    }
};
