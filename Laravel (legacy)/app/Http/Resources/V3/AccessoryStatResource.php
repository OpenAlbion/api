<?php

namespace App\Http\Resources\V3;

use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class AccessoryStatResource extends JsonResource
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
            'accessory' => AccessoryResource::make($this->whenLoaded('accessory')),
            'stats' => $this->stats,
        ];
    }
}
