<?php

namespace App\Services\Wiki\Concerns\Browsershot;

use Spatie\Browsershot\Browsershot;

trait CanBuildRequest
{
    public function buildRequest(): Browsershot
    {
        return new Browsershot();
    }
}
