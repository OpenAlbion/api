<?php

namespace App\Exceptions;

use Illuminate\Foundation\Exceptions\Handler as ExceptionHandler;
use Throwable;
use Spatie\SlackAlerts\Facades\SlackAlert;

class Handler extends ExceptionHandler
{
    /**
     * The list of the inputs that are never flashed to the session on validation exceptions.
     *
     * @var array<int, string>
     */
    protected $dontFlash = [
        'current_password',
        'password',
        'password_confirmation',
    ];

    /**
     * Register the exception handling callbacks for the application.
     */
    public function register(): void
    {
        $this->reportable(function (Throwable $e) {
            if (config('slack-alerts.webhook_urls.error')) {
                SlackAlert::to('error')->message(
                    "*{$e->getMessage()}*\n"
                    . "File: `{$e->getFile()}`\n"
                    . "Line: `{$e->getLine()}`\n"
                );
            }
        });
    }
}
