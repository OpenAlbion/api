<?php

namespace App\Services\Concerns;

use GuzzleHttp\Client;

trait BuildRequest
{
    public function buildRequest(): Client
    {
        return $this->withBaseUrl();
    }

    public function withBaseUrl(): Client
    {
        return new Client([
            'base_uri' => $this->baseUrl,
        ]);
    }
}
