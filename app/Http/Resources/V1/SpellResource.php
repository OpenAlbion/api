<?php

namespace App\Http\Resources\V1;

use App\Services\Render\RenderService;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class SpellResource extends JsonResource
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
            'slot' => $this->slot->slotName(),
            'preview' => $this->preview,
            'icon' => app(RenderService::class)->renderSpell($this->identifier),
            'description' => strip_tags($this->description),
            'description_html' => $this->description,
        ];
    }
}
