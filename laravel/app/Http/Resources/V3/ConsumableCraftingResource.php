<?php

namespace App\Http\Resources\V3;

use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class ConsumableCraftingResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'id' => $this->id,
            'per_craft' => $this->per_craft,
            'enchantment' => $this->enchantment,
            'consumable' => ConsumableResource::make($this->whenLoaded('consumable')),
            'requirements' => $this->requirements,
        ];
    }
}
