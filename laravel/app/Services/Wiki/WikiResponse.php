<?php

namespace App\Services\Wiki;

use App\Services\Wiki\Contracts\WikiResponseInterface;
use Psr\Http\Message\ResponseInterface;
use Spatie\Browsershot\Browsershot;

class WikiResponse implements WikiResponseInterface
{
    public function __construct(
        private readonly ResponseInterface|Browsershot $response
    ) {
    }

    public function toHtml(): string
    {
        if ($this->response instanceof Browsershot) {
            return $this->response->bodyHtml();
        }

        if ($this->response instanceof ResponseInterface) {
            return $this->response
                ->getBody()
                ->__toString();
        }
    }
}
