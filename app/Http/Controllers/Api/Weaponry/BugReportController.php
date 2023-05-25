<?php

namespace App\Http\Controllers\Api\Weaponry;

use App\Http\Controllers\Controller;
use App\Http\Requests\BugReportRequest;
use Illuminate\Http\Request;
use Spatie\SlackAlerts\Facades\SlackAlert;

class BugReportController extends Controller
{
    public function report(BugReportRequest $request)
    {
        $debug = json_encode($request->validated('debug'), JSON_PRETTY_PRINT);
        SlackAlert::to('application')->message(
            "*{$request->validated('category')}*\n"
            . "{$request->validated('description')}\n"
            . "```{$debug}```"
        );
        return response()->json(['message' => 'success']);
    }
}
