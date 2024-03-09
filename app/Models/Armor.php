<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Laravel\Scout\Searchable;
use Illuminate\Database\Eloquent\Casts\Attribute;

class Armor extends Model
{
    use HasFactory;
    use Searchable;

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

    public function subcategory(): BelongsTo
    {
        return $this->belongsTo(Category::class, 'subcategory_id');
    }

    public function spells(): BelongsToMany
    {
        return $this->belongsToMany(Spell::class, 'weapon_spells');
    }

    public function armorStats(): HasMany
    {
        return $this->hasMany(ArmorStat::class);
    }

    protected function tier(): Attribute
    {
        return Attribute::make(
            get: fn ($value) => $value ? number_format($value, 1) : null,
        );
    }
}
