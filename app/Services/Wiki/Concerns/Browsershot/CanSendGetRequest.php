<?php

namespace App\Services\Wiki\Concerns\Browsershot;

use App\Services\Wiki\Contracts\WikiResponseInterface;
use App\Services\Wiki\WikiResponse;
use Spatie\Browsershot\Browsershot;

trait CanSendGetRequest
{
    public function get(Browsershot $request, string $url): WikiResponseInterface
    {
        return new WikiResponse($request->url($this->baseUrl.$url));
    }
}
