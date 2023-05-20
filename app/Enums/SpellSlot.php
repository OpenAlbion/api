<?php

namespace App\Enums;

enum SpellSlot: string
{
    case Q = 'q';
    case W = 'w';
    case E = 'e';
    case D = 'd';
    case R = 'r';
    case F = 'f';
    case PASSIVE = 'passive';

    public function slotName(): string
    {
        return match ($this) {
            SpellSlot::Q => 'First Slot',
            SpellSlot::W => 'Second Slot',
            SpellSlot::E => 'Third Slot',
            SpellSlot::PASSIVE => 'Passive',
        };
    }
}
