<?php

namespace App\Http\Resources\V2;

use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class WeaponStatResource extends JsonResource
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
            'quality' => $this->quality,
            'enchantment' => $this->enchantment,
            'weapon' => WeaponResource::make($this->whenLoaded('weapon')),
            'stats' => $this->stats,
        ];
    }
}
