<?php

namespace App\Services\Wiki\Concerns\Guzzle;

use App\Services\Wiki\Contracts\WikiResponseInterface;
use App\Services\Wiki\WikiResponse;
use GuzzleHttp\Client;

trait CanSendGetRequest
{
    public function get(Client $request, string $url): WikiResponseInterface
    {
        return new WikiResponse($request->get($url));
    }
}
