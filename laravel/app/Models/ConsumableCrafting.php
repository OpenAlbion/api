<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class ConsumableCrafting extends Model
{
    use HasFactory;

    protected $fillable = [
        'consumable_id',
        'per_craft',
        'enchantment',
        'requirements',
    ];

    protected $casts = [
        'requirements' => 'array',
    ];

    public function consumable()
    {
        return $this->belongsTo(Consumable::class);
    }
}
