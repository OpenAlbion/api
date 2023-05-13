<?php

namespace App\Enums;

enum SpellSlot: string
{
    case Q = 'q';
    case W = 'w';
    case E = 'e';
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
