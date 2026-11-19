<?php

namespace App\Http\Resources\V1;

use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class ArmorStatResource extends JsonResource
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
            'armor' => ArmorResource::make($this->whenLoaded('armor')),
            'stats' => $this->stats,
        ];
    }
}
