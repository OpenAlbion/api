<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class ConsumableStat extends Model
{
    use HasFactory;

    protected $fillable = [
        'consumable_id',
        'quality',
        'enchantment',
        'stats',
    ];

    protected $casts = [
        'stats' => 'array',
    ];

    public function consumable()
    {
        return $this->belongsTo(Consumable::class);
    }
}
