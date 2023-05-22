<?php

namespace App\Http\Resources\Weaponry;

use App\Enums\CategoryType;
use App\Models\Accessory;
use App\Models\Armor;
use App\Models\Weapon;
use App\Services\Render\RenderService;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class SearchResource extends JsonResource
{
    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        $type = null;
        if ($this->resource instanceof Weapon) {
            $type = CategoryType::WEAPON;
        } elseif ($this->resource instanceof Armor) {
            $type = CategoryType::ARMOR;
        } elseif ($this->resource instanceof Accessory) {
            $type = CategoryType::ACCESSORY;
        }
        return [
            'type' => $type,
            'type_id' => $this->id,
            'name' => $this->name,
            'tier' => $this->tier,
            'item_power' => $this->item_power,
            'identifier' => $this->identifier,
            'icon' => app(RenderService::class)->renderItem($this->identifier),
        ];
    }
}
