<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class AccessoryStat extends Model
{
    use HasFactory;

    protected $fillable = [
        'accessory_id',
        'quality',
        'enchantment',
        'stats',
    ];

    protected $casts = [
        'stats' => 'array',
    ];

    public function accessory()
    {
        return $this->belongsTo(Accessory::class);
    }
}
