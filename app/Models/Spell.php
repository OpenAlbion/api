<?php

namespace App\Models;

use App\Enums\SpellSlot;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;

class Spell extends Model
{
    use HasFactory;

    protected $fillable = [
        'name',
        'identifier',
        'slot',
        'attributes',
        'description',
        'preview',
        'ref',
    ];

    protected $casts = [
        'attributes' => 'array',
        'slot' => SpellSlot::class,
    ];

    public function weaponSpells()
    {
        return $this->hasMany(WeaponSpell::class);
    }
}
