<?php

namespace App\Services\Wiki\Contracts;

interface WikiResponseInterface
{
    public function toHtml(): string;
}
