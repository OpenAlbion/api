<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class WeaponStat extends Model
{
    use HasFactory;

    protected $fillable = [
        'weapon_id',
        'quality',
        'enchantment',
        'stats',
    ];

    protected $casts = [
        'stats' => 'array',
    ];

    public function weapon()
    {
        return $this->belongsTo(Weapon::class);
    }
}
