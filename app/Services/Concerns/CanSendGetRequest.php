<?php

namespace App\Services\Concerns;

use GuzzleHttp\Client;
use Psr\Http\Message\ResponseInterface;

trait CanSendGetRequest
{
    public function get(Client $request, string $url): ResponseInterface
    {
        return $request->get($url);
    }
}
