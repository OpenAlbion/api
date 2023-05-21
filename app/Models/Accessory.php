<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;
use Illuminate\Database\Eloquent\Relations\HasMany;

class Accessory extends Model
{
    use HasFactory;

    protected $fillable = [
        'category_id',
        'subcategory_id',
        'name',
        'identifier',
        'tier',
        'item_power',
        'path',
    ];

    public function category(): BelongsTo
    {
        return $this->belongsTo(Category::class);
    }

    public function spells(): BelongsToMany
    {
        return $this->belongsToMany(Spell::class, 'accessory_spells');
    }

    public function accessoryStats(): HasMany
    {
        return $this->hasMany(AccessoryStat::class);
    }
}
