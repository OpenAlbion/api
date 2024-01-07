<?php

namespace App\Services\Wiki\Concerns\Guzzle;

use GuzzleHttp\Client;

trait CanBuildRequest
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
