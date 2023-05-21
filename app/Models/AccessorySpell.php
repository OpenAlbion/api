<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class AccessorySpell extends Model
{
    use HasFactory;

    protected $fillable = [
        'accessory_id',
        'spell_id',
    ];

    public function accessory(): BelongsTo
    {
        return $this->belongsTo(Accessory::class);
    }

    public function spell(): BelongsTo
    {
        return $this->belongsTo(Spell::class);
    }
}
