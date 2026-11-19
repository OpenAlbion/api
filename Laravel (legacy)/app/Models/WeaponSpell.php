<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class WeaponSpell extends Model
{
    use HasFactory;

    protected $fillable = [
        'weapon_id',
        'spell_id',
    ];

    public function weapon(): BelongsTo
    {
        return $this->belongsTo(Weapon::class);
    }

    public function spell(): BelongsTo
    {
        return $this->belongsTo(Spell::class);
    }
}
