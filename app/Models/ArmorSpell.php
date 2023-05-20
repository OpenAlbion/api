<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class ArmorSpell extends Model
{
    use HasFactory;

    protected $fillable = [
        'armor_id',
        'spell_id',
    ];

    public function armor(): BelongsTo
    {
        return $this->belongsTo(Armor::class);
    }

    public function spell(): BelongsTo
    {
        return $this->belongsTo(Spell::class);
    }
}
