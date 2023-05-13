<?php

namespace App\Services\Render;

class RenderService
{
    private int $enchantment;

    private int $quality;

    private int $size;

    private string $locale;

    public function __construct()
    {
        $this->enchantment = 0;
        $this->quality = 0;
        $this->size = 217;
        $this->locale = 'en';
    }

    public function renderItem(string $identifier): string
    {
        return "https://render.albiononline.com/v1/item/{$identifier}@{$this->enchantment}.png?quality={$this->quality}&size={$this->size}&locale={$this->locale}";
    }

    public function renderSpell(string $identifier): string
    {
        return "https://render.albiononline.com/v1/spell/{$identifier}.png?size={$this->size}&locale={$this->locale}";
    }

    public function setEnchantment(int $val)
    {
        $this->enchantment = $val;

        return $this;
    }

    public function setQuality(int $val)
    {
        $this->quality = $val;

        return $this;
    }

    public function setSize(int $val)
    {
        $this->quality = $val;

        return $this;
    }

    public function setLocale(int $val)
    {
        $this->quality = $val;

        return $this;
    }
}
