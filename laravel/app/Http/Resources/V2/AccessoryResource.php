<?php

namespace App\Http\Resources\V2;

use App\Services\Render\RenderService;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class AccessoryResource extends JsonResource
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
            'name' => $this->name,
            'tier' => $this->tier,
            'item_power' => $this->item_power,
            'identifier' => $this->identifier,
            'icon' => app(RenderService::class)->renderItem($this->identifier),
        ];
    }
}
